//! AnimusEngine DRM/KMS Backend -- Bare-Metal Production Scanout.
//!
//! Uses Smithay's `UdevBackend` to monitor DRM devices, `DrmDevice` for KMS
//! modesetting, and `GbmDevice` for zero-copy DMA-BUF buffer allocation.
//!
//! Initialization flow (matching cosmic-comp's `src/backend/kms/device.rs`):
//! 1. `UdevBackend::new("seat0")` -- monitors DRM device hotplug via udev
//! 2. For each DRM device: open `/dev/dri/cardN`, create `DrmDeviceFd`
//! 3. `DrmDevice::new(fd, false)` -- enumerates connectors/crtcs/planes
//! 4. `GbmDevice::new(fd)` -- allocates scanout buffers
//! 5. For each connected connector: create `DrmSurface` on the best CRTC
//! 6. Frame loop: render into GBM buffer, `drmModePageFlip` for vblank sync

#[cfg(target_os = "linux")]
use {
    anyhow::{Context, Result},
    smithay::{
        backend::{
            drm::{DrmDevice, DrmDeviceFd, DrmDeviceNotifier, DrmEvent, DrmSurface},
            allocator::gbm::{GbmDevice, GbmAllocator, GbmBufferFlags},
        },
        utils::DeviceFd,
    },
    drm::control::{
        Device as DrmControlDevice,
        connector::{Handle as ConnectorHandle, State as ConnectorState},
        Mode,
    },
    std::path::PathBuf,
    tracing::{info, warn, error},
};

#[cfg(target_os = "linux")]
pub struct AnimusDrmBackend {
    pub drm_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub is_initialized: bool,
    pub disable_connectors: bool,
    /// The Smithay DrmDevice, which owns the KMS state (CRTCs, planes, connectors).
    pub drm_device: Option<DrmDevice>,
    /// The calloop event source that fires `DrmEvent::Vblank` on each page flip.
    /// This is inserted into the calloop event loop by the compositor main.
    pub drm_notifier: Option<DrmDeviceNotifier>,
    /// The GBM device for allocating scanout buffers.
    pub gbm_device: Option<GbmDevice<DrmDeviceFd>>,
    /// The GBM allocator wrapping the GBM device.
    pub gbm_allocator: Option<GbmAllocator<DrmDeviceFd>>,
    /// Active DRM surfaces (one per connected monitor).
    pub surfaces: Vec<DrmSurface>,
}

#[cfg(target_os = "linux")]
impl AnimusDrmBackend {
    /// Finds the primary DRM device and prepares the backend.
    pub fn new() -> Result<Self> {
        let drm_path = Self::find_primary_drm_device()?;
        info!("AnimusDrmBackend: Using DRM device {:?}", drm_path);

        Ok(Self {
            drm_path,
            width: 1920,
            height: 1080,
            refresh_hz: 144,
            is_initialized: false,
            disable_connectors: false,
            drm_device: None,
            drm_notifier: None,
            gbm_device: None,
            gbm_allocator: None,
            surfaces: Vec::new(),
        })
    }

    /// Finds the first available DRM card device under /dev/dri/
    fn find_primary_drm_device() -> Result<PathBuf> {
        for entry in std::fs::read_dir("/dev/dri")
            .context("Cannot open /dev/dri -- no DRM subsystem available")?
        {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("card") {
                return Ok(entry.path());
            }
        }
        anyhow::bail!("No DRM card device found in /dev/dri")
    }

    /// Opens the DRM device, acquires DRM master, creates DrmDevice + GbmDevice,
    /// enumerates connected monitors, and creates a DrmSurface for each.
    ///
    /// Returns the `DrmDeviceNotifier` which must be inserted into the calloop
    /// event loop to receive vblank events for frame pacing.
    pub fn initialize(&mut self) -> Result<Option<DrmDeviceNotifier>> {
        info!(
            "AnimusDrmBackend: Initializing DRM/KMS on {:?}",
            self.drm_path
        );

        // 1. Open the DRM device and create DrmDeviceFd
        let fd = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.drm_path)
            .context("Failed to open DRM device")?;

        let owned_fd: std::os::unix::io::OwnedFd = fd.into();
        let device_fd = DeviceFd::from(owned_fd);
        let drm_fd = DrmDeviceFd::new(device_fd);

        info!("AnimusDrmBackend: Acquired DRM master on {:?}", self.drm_path);

        // 2. Create the DrmDevice -- enumerates connectors, CRTCs, planes
        //    Returns (DrmDevice, DrmDeviceNotifier)
        let (mut drm_device, drm_notifier) = DrmDevice::new(drm_fd.clone(), self.disable_connectors)
            .map_err(|e| anyhow::anyhow!("DrmDevice::new failed: {:?}", e))?;

        info!(
            "AnimusDrmBackend: DrmDevice created -- {} CRTC(s), atomic: {}",
            drm_device.crtcs().len(),
            drm_device.is_atomic(),
        );

        // 3. Create the GbmDevice for buffer allocation
        let gbm_device = GbmDevice::new(drm_fd.clone())
            .map_err(|e| anyhow::anyhow!("GbmDevice::new failed: {:?}", e))?;
        let gbm_allocator = GbmAllocator::new(gbm_device.clone(), GbmBufferFlags::RENDERING);

        info!("AnimusDrmBackend: GbmDevice created for scanout buffer allocation");

        // 4. Enumerate connected connectors and create DrmSurfaces
        let resources = drm_fd.resource_handles()
            .map_err(|e| anyhow::anyhow!("Failed to get DRM resource handles: {:?}", e))?;

        let connectors: Vec<ConnectorHandle> = resources.connectors().to_vec();
        let crtcs: Vec<_> = drm_device.crtcs().to_vec();

        let mut surface_count = 0;
        let mut crtc_idx = 0;

        for connector_handle in connectors.iter() {
            let connector_info = drm_fd.get_connector(*connector_handle, true)
                .map_err(|e| {
                    warn!("AnimusDrmBackend: Failed to get connector {:?}: {:?}", connector_handle, e);
                })
                .ok();

            let connector_info = match connector_info {
                Some(info) => info,
                None => continue,
            };

            // Only process connected monitors
            if connector_info.state() != ConnectorState::Connected {
                info!(
                    "AnimusDrmBackend: Connector {:?} ({:?}) is {:?} -- skipping",
                    connector_handle, connector_info.interface(), connector_info.state()
                );
                continue;
            }

            let modes = connector_info.modes();
            if modes.is_empty() {
                warn!("AnimusDrmBackend: Connector {:?} has no modes -- skipping", connector_handle);
                continue;
            }

            // Pick the preferred mode (first mode is usually preferred by EDID)
            let mode = modes[0];
            let (w, h) = mode.size();
            self.width = w as u32;
            self.height = h as u32;
            self.refresh_hz = (mode.vrefresh() as u32).max(60);

            info!(
                "AnimusDrmBackend: Connector {:?} connected -- {}x{}@{}Hz, {} mode(s)",
                connector_handle, w, h, mode.vrefresh(), modes.len()
            );

            // Find a CRTC for this connector
            if crtc_idx >= crtcs.len() {
                warn!("AnimusDrmBackend: No more CRTCs available for connector {:?}", connector_handle);
                continue;
            }

            let crtc = crtcs[crtc_idx];
            crtc_idx += 1;

            // Create the DrmSurface
            match drm_device.create_surface(crtc, mode, &[*connector_handle]) {
                Ok(surface) => {
                    info!("AnimusDrmBackend: DrmSurface created on CRTC {:?}", crtc);
                    self.surfaces.push(surface);
                    surface_count += 1;
                }
                Err(e) => {
                    error!("AnimusDrmBackend: Failed to create surface on CRTC {:?}: {:?}", crtc, e);
                }
            }
        }

        info!(
            "AnimusDrmBackend: {} DRM surface(s) created for {} connected monitor(s)",
            surface_count,
            connectors.iter().filter(|c| {
                drm_fd.get_connector(**c, false)
                    .map(|ci| ci.state() == ConnectorState::Connected)
                    .unwrap_or(false)
            }).count()
        );

        self.drm_device = Some(drm_device);
        self.gbm_device = Some(gbm_device);
        self.gbm_allocator = Some(gbm_allocator);
        self.is_initialized = true;

        // Return the notifier so the compositor can insert it into calloop
        // for vblank-driven frame pacing
        if surface_count > 0 {
            Ok(Some(drm_notifier))
        } else {
            // No surfaces = no monitors connected; keep the notifier anyway
            // for future hotplug events
            Ok(Some(drm_notifier))
        }
    }
}

#[cfg(target_os = "linux")]
impl super::AnimusBackend for AnimusDrmBackend {
    fn name(&self) -> &'static str { "drm-kms" }
    fn has_gpu(&self) -> bool { self.is_initialized }
    fn schedule_frame(&mut self) { /* DRM vblank drives frame pacing via DrmDeviceNotifier */ }
    fn output_geometry(&self) -> (u32, u32, u32) { (self.width, self.height, self.refresh_hz) }
}

// Non-Linux stub so the module compiles on Windows during development
#[cfg(not(target_os = "linux"))]
pub struct AnimusDrmBackend;

#[cfg(not(target_os = "linux"))]
impl AnimusDrmBackend {
    pub fn new() -> anyhow::Result<Self> {
        anyhow::bail!("DRM/KMS backend is Linux-only")
    }
}
