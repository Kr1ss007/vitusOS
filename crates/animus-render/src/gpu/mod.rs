//! GPU Render Backend Abstraction for AnimusEngine.
//!
//! Routes rendering to the appropriate GPU backend:
//! - `AnimusVulkanRenderer` — Production Vulkan 1.3 pipeline (Linux, bare metal)
//! - `ScanoutFramebuffer` — CPU software rasterizer (WSL2/fallback)
//!
//! The architecture:
//! ```
//! Smithay compositor → GPU mod → AnimusVulkanRenderer → vkQueueSubmit → DRM page flip
//!                                      ↑
//!                              shaderc (GLSL→SPIR-V)
//!                              ash (raw Vulkan calls)
//!                              ScanoutFramebuffer (CPU compositing)
//! ```

pub mod vulkan;
pub use vulkan::AnimusVulkanRenderer;
