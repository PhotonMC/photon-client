//! Vulkan device and swapchain set up the way the reference's Vulkan backend
//! does it.
//!
//! Nothing here draws world content yet: the backend comes up, acquires a
//! swapchain image, clears it and presents. That is enough to prove the surface
//! is live, and it deliberately makes no parity claim about pixels — those
//! start once the shader pipeline exists and P15.a has measured what the Java
//! reference's own repeats vary by.
//!
//! The choices that *are* taken from the reference are the ones that decide
//! which GPU and which image the client ends up with: API version, required
//! device extensions and features, discrete-GPU preference, surface format
//! filter, and the swapchain parameters.
//!
//! # Safety conventions
//!
//! Every `unsafe` block here is a call into Vulkan through `ash`, and they all
//! lean on the same few invariants of [`VulkanBackend`]. They are stated once,
//! and each block names the ones it needs.
//!
//! - **Live handles.** `instance`, `device`, `surface`, the swapchain, the
//!   queue, the command pool and the command buffer are made by `create` and
//!   destroyed only in `Drop`. Every other handle sits in a field that owns it
//!   and is taken out of that field before it is destroyed. A handle read from
//!   `self` is therefore valid, and none is destroyed twice.
//!   Where a type owns its handles and frees them when dropped (`HostBuffer`, `Image`, `Sampler`, `Pipeline`), the
//!   backend's `Drop` releases the owners while the device is still live.
//! - **Idle GPU.** A resource is destroyed or overwritten only after the work
//!   that may read it has finished: those frames' fences were waited on (`complete_through`),
//!   or the device was made idle. Up to `FRAMES_IN_FLIGHT` frames are in flight, and every frame
//!   is numbered; a resource records the frames that may use it as such a number.
//! - **Recording.** Commands go to `command_buffer` between the begin and the
//!   end in `render_frame`.
//!
//! The public methods that use the queue or the command buffer take
//! `&mut self`, so those are never used from two places at once. A private
//! `unsafe fn` states in its `# Safety` section what its caller must add to
//! these; the public methods that call it do.

use std::ffi::{CStr, CString};

use ash::vk;

use crate::frame_timing::{BackendTimings, GpuTimings, TIMESTAMP_COUNT};
use crate::scene::{BlockVertex, Mesh, Vertex};
use crate::texture::png::DecodedImage;

mod bytes;
mod device;
mod draws;
mod frame;
mod memory;
mod owned;
mod pipeline;
mod pipeline_cache;
mod scene;
mod sections;
mod shaders;
mod swapchain;
mod texture;
mod validation;

use bytes::as_bytes;
pub use draws::DrawKind;
use frame::transition;
use owned::{BoundBuffer, Descriptors, Image, Pipeline, Sampler, Texture};
use swapchain::{choose_present_mode, clamp_extent};
pub use validation::validation_findings;

mod buffer;
pub use buffer::*;
mod backend;
pub use backend::*;
