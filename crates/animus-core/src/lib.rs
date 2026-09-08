pub mod ae_bridge;
pub mod accessibility;
pub mod clipboard;
pub mod context;
pub mod crash;
pub mod dbus;
pub mod drag;
pub mod engine;
pub mod eobus;
pub mod event_bus;
pub mod events;
pub mod handoff;
pub mod hardware;
pub mod portal;
pub mod power;
pub mod registry;
pub mod sound;
pub mod state;

pub use ae_bridge::{AEBridge, BridgedEvent};
pub use accessibility::{AccessibilityProvider, A11yNode, A11yRole, A11yState};
pub use clipboard::ClipboardBridge;
pub use context::{AnimusContext, ContextOriginType};
pub use crash::{CrashManager, CrashSite, FirstResponder, GlobalFeed, Handshakes, PressureLevel, ResourceSnapshot, SubsystemHealth, Vessel, VesselState, Vessels};
pub use dbus::{AudioDbusClient, BluetoothDbusClient, LogindDbusClient, NetworkDbusClient, SystemDbusManager};
pub use drag::{DragManager, DragPayload, DragPayloadType};
pub use engine::AnimusEngine;
pub use eobus::{EOBus, OutsiderStatus, DBusBridge};
pub use event_bus::EventBus;
pub use events::{AEEvent, DragPayload as AEDragPayload, DragPayloadType as AEDragPayloadType, NotificationPayload};
pub use handoff::{AnimusGpuHandoff, GpuType as HandoffGpuType, GpuVendor as HandoffGpuVendor, ANIMUS_HANDOFF_GUID_STR};
pub use hardware::{GpuDeviceInfo, GpuType, GpuVendor, HardwareTopology};
pub use portal::{PortalGateway, PortalRequest};
pub use power::{LidCloseAction, PowerManager};
pub use registry::{
    ClientRecord, ClientRegistry, LiveCounts, NotificationLike, NotificationRegistry,
    RegHandle, REG_INVALID, RegistryManager, RegistrySchema, RegistryValue,
    SurfaceRegistry, WindowLike, WindowRegistry,
};
pub use sound::{AudioBackend, AudioSinkInfo, SoundEngine, sounds};
pub use state::{StateManager, state_keys};

