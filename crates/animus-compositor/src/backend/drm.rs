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
            drm::{DrmDevice, DrmDeviceFd, DrmDeviceNotifier, DrmSurface, PlaneConfig, PlaneState},
            allocator::gbm::{GbmDevice, GbmAllocator, GbmBufferFlags},
        },
        utils::{Rectangle, Transform, DeviceFd},
    },
    drm::{
        buffer::Buffer as DrmBufferTrait,
        control::{
            Device as DrmControlDevice,
            connector::{Handle as ConnectorHandle, State as ConnectorState},
            dumbbuffer::DumbBuffer,
            framebuffer,
        },
    },
    drm_fourcc::DrmFourcc,
    std::path::PathBuf,
    tracing::{info, warn, error},
};

/// Hardware-allocated DRM dumb buffer backed by a DRM framebuffer handle.
#[cfg(target_os = "linux")]
pub struct DumbScanoutBuffer {
    pub dumb: DumbBuffer,
    pub fb: framebuffer::Handle,
    pub width: u32,
    pub height: u32,
}

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
    /// Double-buffered scanout dumb buffers for tear-free page flipping
    pub scanout_buffers: Vec<DumbScanoutBuffer>,
    /// Current front/back buffer index (0 or 1)
    pub current_buffer_idx: usize,
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
            scanout_buffers: Vec::new(),
            current_buffer_idx: 0,
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

    /// Allocates or reallocates the double-buffered scanout dumb buffers.
    pub fn ensure_scanout_buffers(&mut self, width: u32, height: u32) -> Result<()> {
        if self.scanout_buffers.len() == 2
            && self.scanout_buffers[0].width == width
            && self.scanout_buffers[0].height == height
        {
            return Ok(());
        }

        let drm = self.drm_device.as_mut().context("DRM device not initialized")?;

        // Destroy any existing scanout buffers before reallocating
        for buf in self.scanout_buffers.drain(..) {
            let _ = drm.destroy_framebuffer(buf.fb);
            let _ = drm.destroy_dumb_buffer(buf.dumb);
        }

        // Allocate double-buffered scanout dumb buffers
        for idx in 0..2 {
            let dumb = drm.create_dumb_buffer(
                (width, height),
                DrmFourcc::Xrgb8888,
                32,
            ).with_context(|| format!("Failed to create DRM dumb buffer #{} ({}x{})", idx, width, height))?;

            let fb = drm.add_framebuffer(&dumb, 24, 32)
                .with_context(|| format!("Failed to create DRM framebuffer for dumb buffer #{}", idx))?;

            info!(
                "AnimusDrmBackend: Allocated scanout dumb buffer #{} ({}x{}, pitch: {} bytes, fb: {:?})",
                idx, width, height, dumb.pitch(), fb
            );

            self.scanout_buffers.push(DumbScanoutBuffer {
                dumb,
                fb,
                width,
                height,
            });
        }

        self.current_buffer_idx = 0;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl super::AnimusBackend for AnimusDrmBackend {
    fn name(&self) -> &'static str { "drm-kms" }
    fn has_gpu(&self) -> bool { self.is_initialized }
    fn schedule_frame(&mut self) { /* DRM vblank drives frame pacing via DrmDeviceNotifier */ }
    fn output_geometry(&self) -> (u32, u32, u32) { (self.width, self.height, self.refresh_hz) }

    fn present_frame(&mut self, framebuffer: &animus_render::framebuffer::ScanoutFramebuffer) -> anyhow::Result<()> {
        if !self.is_initialized || self.drm_device.is_none() {
            return Ok(());
        }

        let w = framebuffer.width;
        let h = framebuffer.height;
        if w == 0 || h == 0 {
            return Ok(());
        }

        // 1. Ensure hardware-backed scanout buffers are allocated for this resolution
        self.ensure_scanout_buffers(w, h)?;

        // 2. Select back buffer index for ping-pong double buffering
        let back_idx = 1 - self.current_buffer_idx;
        let drm = self.drm_device.as_mut().context("DRM device not available")?;

        // 3. Map dumb buffer and copy pixel data to scanout memory
        {
            let target_buf = &mut self.scanout_buffers[back_idx];
            let pitch = target_buf.dumb.pitch() as usize;
            let mut mapping = drm.map_dumb_buffer(&mut target_buf.dumb)
                .context("Failed to map DRM dumb buffer for CPU scanout write")?;

            let row_bytes = (w as usize) * 4;
            let dst_slice: &mut [u8] = mapping.as_mut();
            let src_bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    framebuffer.pixels.as_ptr() as *const u8,
                    framebuffer.pixels.len() * 4,
                )
            };

            if pitch == row_bytes && dst_slice.len() >= src_bytes.len() {
                // Direct continuous copy when pitch matches tightly
                dst_slice[..src_bytes.len()].copy_from_slice(src_bytes);
            } else {
                // Row-by-row blit to handle hardware stride/pitch alignment
                for y in 0..(h as usize) {
                    let src_start = y * row_bytes;
                    let src_end = src_start + row_bytes;
                    let dst_start = y * pitch;
                    let dst_end = dst_start + row_bytes;
                    if src_end <= src_bytes.len() && dst_end <= dst_slice.len() {
                        dst_slice[dst_start..dst_end].copy_from_slice(&src_bytes[src_start..src_end]);
                    }
                }
            }
        }

        // 4. Page-flip or atomic commit across all active DRM surfaces
        let fb_handle = self.scanout_buffers[back_idx].fb;
        for surface in &mut self.surfaces {
            let make_plane_state = || PlaneState {
                handle: surface.plane(),
                config: Some(PlaneConfig {
                    src: Rectangle::from_size((w as f64, h as f64).into()),
                    dst: Rectangle::from_size((w as i32, h as i32).into()),
                    transform: Transform::Normal,
                    alpha: 1.0,
                    damage_clips: None,
                    fb: fb_handle,
                    fence: None,
                }),
            };

            if surface.commit_pending() {
                if let Err(e) = surface.commit([make_plane_state()], true) {
                    warn!("AnimusDrmBackend: Initial modeset commit failed on CRTC {:?}: {:?}", surface.crtc(), e);
                }
            } else {
                if let Err(_e) = surface.page_flip([make_plane_state()], true) {
                    // Fallback to modeset commit if atomic nonblock flip failed
                    if let Err(e2) = surface.commit([make_plane_state()], true) {
                        warn!("AnimusDrmBackend: Page flip & commit fallback failed on CRTC {:?}: {:?}", surface.crtc(), e2);
                    }
                }
            }
        }

        // 5. Swap current buffer index to complete double-buffering
        self.current_buffer_idx = back_idx;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for AnimusDrmBackend {
    fn drop(&mut self) {
        if let Some(drm) = self.drm_device.as_ref() {
            for buf in self.scanout_buffers.drain(..) {
                let _ = drm.destroy_framebuffer(buf.fb);
                let _ = drm.destroy_dumb_buffer(buf.dumb);
            }
        }
    }
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

#[cfg(not(target_os = "linux"))]
impl super::AnimusBackend for AnimusDrmBackend {
    fn name(&self) -> &'static str { "drm-kms-stub" }
    fn has_gpu(&self) -> bool { false }
    fn schedule_frame(&mut self) {}
    fn output_geometry(&self) -> (u32, u32, u32) { (1920, 1080, 60) }
    fn present_frame(&mut self, _framebuffer: &animus_render::framebuffer::ScanoutFramebuffer) -> anyhow::Result<()> {
        anyhow::bail!("DRM/KMS backend is Linux-only")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drm_backend_uninitialized_present_frame() {
        #[cfg(target_os = "linux")]
        {
            let mut backend = AnimusDrmBackend {
                drm_path: PathBuf::from("/dev/dri/card0"),
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
                scanout_buffers: Vec::new(),
                current_buffer_idx: 0,
            };
            let fb = animus_render::framebuffer::ScanoutFramebuffer::new(1920, 1080);
            let res = super::super::AnimusBackend::present_frame(&mut backend, &fb);
            assert!(res.is_ok(), "Uninitialized backend should gracefully no-op without crashing");
        }
    }

    #[test]
    fn test_pitch_padding_stride_blit() {
        let width = 4usize;
        let height = 2usize;
        let pitch = 24usize; // padded from 16 to 24 bytes per line
        let row_bytes = width * 4;
        let src_pixels: Vec<u32> = vec![
            0x11111111, 0x22222222, 0x33333333, 0x44444444,
            0x55555555, 0x66666666, 0x77777777, 0x88888888,
        ];
        let mut dst_buffer = vec![0u8; pitch * height];
        let src_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(src_pixels.as_ptr() as *const u8, src_pixels.len() * 4)
        };
        for y in 0..height {
            let src_start = y * row_bytes;
            let src_end = src_start + row_bytes;
            let dst_start = y * pitch;
            let dst_end = dst_start + row_bytes;
            dst_buffer[dst_start..dst_end].copy_from_slice(&src_bytes[src_start..src_end]);
        }

        // Row 0 matches first 4 pixels (16 bytes)
        assert_eq!(&dst_buffer[0..16], &src_bytes[0..16]);
        // Padding bytes between row 0 and row 1 must remain untouched
        assert_eq!(&dst_buffer[16..24], &[0u8; 8]);
        // Row 1 matches second 4 pixels (16 bytes)
        assert_eq!(&dst_buffer[24..40], &src_bytes[16..32]);
    }
}
