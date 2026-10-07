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
//!   Where a type owns its handles and frees them when dropped (`HostBuffer`), the
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
use crate::png::DecodedImage;
use crate::scene::{BlockVertex, Mesh, Vertex};

mod bytes;
mod device;
mod draws;
mod frame;
mod memory;
mod pipeline;
mod pipeline_cache;
mod scene;
mod sections;
mod shaders;
mod swapchain;
mod texture;
mod validation;

use bytes::as_bytes;
use frame::transition;
use swapchain::clamp_extent;
pub use validation::validation_findings;

/// No terrain fog: every distance out of reach, as the reference's empty
/// fog buffer.
const NO_FOG: [f32; 8] = [0.0, 0.0, 0.0, 1.0, f32::MAX, f32::MAX, f32::MAX, f32::MAX];

/// Lightmap inputs with nothing dimming them: white sky light at full
/// factor, the default block light tint at the reference's base factor, no
/// ambient light, and the default brightness option.
const DEFAULT_LIGHTMAP: [f32; 12] = [
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    0.847_058_83,
    0.549_019_63,
    1.4,
    0.0,
    0.0,
    0.0,
    0.5,
];

/// Which of the drawing pipelines to build.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PipelineKind {
    Terrain,
    Translucent,
    SkyTranslucent,
    SkyOverlay,
    Clouds,
    Weather,
    Entity,
    /// GUI sprites over the finished frame.
    Gui,
    /// The crosshair, inverting what is behind it.
    GuiInvert,
    /// Items in GUI slots, over a cleared depth.
    GuiItem,
    /// The aimed block's outline.
    Lines,
    /// The crack over a block being broken.
    Crumbling,
    /// The held items, which are drawn with the terrain's shaders but from
    /// the wider vertices the entities use.
    Held,
    /// Text in the world (`TEXT`): the font's glyphs lit by the lightmap and
    /// fogged, tested against the depth.
    Text,
    /// Text seen through blocks (`TEXT_SEE_THROUGH`): no light, fog or depth.
    TextSeeThrough,
    /// World text's shaders over the entity textures, blended and not written
    /// to the depth (`ENTITY_TRANSLUCENT_CULL`): an experience orb.
    TextTranslucent,
    /// An entity's shadow (`ENTITY_SHADOW`): the entity textures, blended, no
    /// light, tested but not written to the depth.
    Shadow,
    /// An energy swirl (`ENERGY_SWIRL`): the entity textures moved and wrapped,
    /// added to what is behind, written to the depth.
    Swirl,
    /// An enchantment's glint (`GLINT`): over the depth the model wrote, blended by its colour. The
    /// reference tests the depth for equality; the same triangles pass the reversed depth's
    /// greater-or-equal test exactly where the model is the nearest surface, which a second
    /// shader's depth (not bit for bit the model's) cannot be trusted to equal.
    Glint,
    /// A dissolving entity (`ENTITY_CUTOUT_DISSOLVE`): the entity's shaders, each
    /// texel dropped by a mask.
    Dissolve,
    /// The dying dragon's rays (`DRAGON_RAYS`): coloured triangles added to what is behind.
    Rays,
}

/// Instance API version the reference asks for.
const API_VERSION: u32 = vk::API_VERSION_1_2;
/// Lowest device API version the reference's feature query accepts.
const MINIMUM_DEVICE_API_VERSION: u32 = vk::API_VERSION_1_1;
/// No transform, for the frame paths that draw nothing.
const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Swapchain images the reference requests, clamped up by surface capabilities.
const REQUESTED_IMAGE_COUNT: u32 = 3;

/// Device extensions the reference requires before a GPU is considered usable.
const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 5] = [
    c"VK_KHR_dynamic_rendering",
    c"VK_KHR_push_descriptor",
    c"VK_KHR_synchronization2",
    c"VK_EXT_vertex_attribute_divisor",
    c"VK_KHR_swapchain",
];

/// Surface formats the reference accepts, paired with an sRGB non-linear space.
const ACCEPTED_SURFACE_FORMATS: [vk::Format; 2] =
    [vk::Format::R8G8B8A8_UNORM, vk::Format::B8G8R8A8_UNORM];

/// Everything that can stop the backend from coming up.
#[derive(Debug)]
pub enum VulkanError {
    /// The loader or a driver entry point was missing.
    Loader(String),
    /// A Vulkan call returned a failure code.
    Call(&'static str, vk::Result),
    /// No physical device met the reference's requirements.
    NoSuitableDevice,
    /// The surface offered no format the reference would accept.
    NoSuitableSurfaceFormat,
    /// The scene was handed an atlas with no levels at all.
    NoAtlasLevels,
}

impl std::fmt::Display for VulkanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loader(detail) => write!(formatter, "Vulkan loader unavailable: {detail}"),
            Self::Call(what, result) => write!(formatter, "{what} failed: {result:?}"),
            Self::NoSuitableDevice => {
                formatter.write_str("no physical device meets the required extensions and features")
            }
            Self::NoSuitableSurfaceFormat => {
                formatter.write_str("surface offers no accepted format in sRGB non-linear space")
            }
            Self::NoAtlasLevels => formatter.write_str("atlas has no mip levels to upload"),
        }
    }
}

impl std::error::Error for VulkanError {}

/// What the backend selected, for reporting without exposing raw handles.
#[derive(Debug, Clone)]
pub struct BackendDescription {
    /// Device name as the driver reports it.
    pub device_name: String,
    /// Whether the selected device is discrete, which the reference prefers.
    pub discrete: bool,
    /// Device API version, encoded as Vulkan does.
    pub api_version: u32,
    /// Chosen swapchain image format.
    pub surface_format: vk::Format,
    /// Number of swapchain images actually created.
    pub image_count: u32,
    /// Swapchain extent in pixels.
    pub extent: vk::Extent2D,
}

/// The query pool the scene's pass timestamps are written to, and what turns
/// its ticks into time.
struct TimestampQueries {
    pool: vk::QueryPool,
    /// Nanoseconds one tick lasts.
    period_ns: f64,
    /// How many bits of a tick the queue writes.
    valid_bits: u32,
    /// Whether each slot's last frame wrote a scene's timestamps not yet read.
    pending: [bool; FRAMES_IN_FLIGHT],
    /// The last frame read back.
    last: Option<GpuTimings>,
}

/// Depth attachment matching the swapchain extent.
struct DepthTarget {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    format: vk::Format,
}

/// Everything needed to draw one textured mesh.
struct SceneResources {
    texture: vk::Image,
    texture_memory: vk::DeviceMemory,
    texture_view: vk::ImageView,
    sampler: vk::Sampler,
    descriptor_pool: vk::DescriptorPool,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    translucent_pipeline_layout: vk::PipelineLayout,
    translucent_pipeline: vk::Pipeline,
    sky_pipeline_layout: vk::PipelineLayout,
    sky_pipeline: vk::Pipeline,
    sky_overlay_pipeline_layout: vk::PipelineLayout,
    sky_overlay_pipeline: vk::Pipeline,
    cloud_pipeline_layout: vk::PipelineLayout,
    cloud_pipeline: vk::Pipeline,
    weather_pipeline_layout: vk::PipelineLayout,
    weather_pipeline: vk::Pipeline,
    /// The rain and snow textures, once loaded, with their sampler and sets.
    weather_textures: Vec<(vk::Image, vk::DeviceMemory, vk::ImageView)>,
    weather_sampler: Option<vk::Sampler>,
    weather_sets: Vec<vk::DescriptorSet>,
    entity_pipeline_layout: vk::PipelineLayout,
    entity_pipeline: vk::Pipeline,
    lines_pipeline_layout: vk::PipelineLayout,
    lines_pipeline: vk::Pipeline,
    crumbling_pipeline_layout: vk::PipelineLayout,
    crumbling_pipeline: vk::Pipeline,
    held_pipeline_layout: vk::PipelineLayout,
    held_pipeline: vk::Pipeline,
    text_pipeline_layout: vk::PipelineLayout,
    text_pipeline: vk::Pipeline,
    text_see_through_pipeline_layout: vk::PipelineLayout,
    text_see_through_pipeline: vk::Pipeline,
    text_translucent_pipeline_layout: vk::PipelineLayout,
    text_translucent_pipeline: vk::Pipeline,
    shadow_pipeline_layout: vk::PipelineLayout,
    shadow_pipeline: vk::Pipeline,
    swirl_pipeline_layout: vk::PipelineLayout,
    swirl_pipeline: vk::Pipeline,
    glint_pipeline_layout: vk::PipelineLayout,
    glint_pipeline: vk::Pipeline,
    dissolve_pipeline_layout: vk::PipelineLayout,
    dissolve_pipeline: vk::Pipeline,
    rays_pipeline_layout: vk::PipelineLayout,
    rays_pipeline: vk::Pipeline,
    crack_texture: Option<(vk::Image, vk::DeviceMemory, vk::ImageView)>,
    crack_sampler: Option<vk::Sampler>,
    crack_set: Option<vk::DescriptorSet>,
    /// The skin entities are drawn with.
    entity_texture: Option<(vk::Image, vk::DeviceMemory, vk::ImageView)>,
    entity_sampler: Option<vk::Sampler>,
    entity_set: Option<vk::DescriptorSet>,
    gui_pipeline_layout: vk::PipelineLayout,
    gui_pipeline: vk::Pipeline,
    gui_invert_pipeline_layout: vk::PipelineLayout,
    gui_invert_pipeline: vk::Pipeline,
    gui_item_pipeline_layout: vk::PipelineLayout,
    gui_item_pipeline: vk::Pipeline,
    /// The GUI sprites' atlas.
    gui_texture: Option<(vk::Image, vk::DeviceMemory, vk::ImageView)>,
    gui_sampler: Option<vk::Sampler>,
    gui_set: Option<vk::DescriptorSet>,
    /// The maps' colours and the decoration sprites, packed in one image, and its sampler and set.
    map_texture: Option<(vk::Image, vk::DeviceMemory, vk::ImageView)>,
    map_sampler: Option<vk::Sampler>,
    map_set: Option<vk::DescriptorSet>,
}

/// A rectangle of one atlas level to rewrite.
#[derive(Debug, Clone)]
pub struct AtlasRegion {
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// Tightly packed RGBA8 rows.
    pub rgba: Vec<u8>,
}

/// How the clouds are placed for a frame.
#[derive(Debug, Clone, Copy)]
pub struct CloudFrame {
    /// The camera-centred view-projection.
    pub view_projection: [[f32; 4]; 4],
    /// The camera's cell corner at the clouds' bottom, from the camera.
    pub offset: [f32; 3],
    /// Where cloud fog is complete, in blocks.
    pub fog_end: f32,
    /// The cloud colour, 0 to 1 per channel.
    pub colour: [f32; 4],
}

/// The clouds' faces, rebuilt when the camera changes cell, and how they are
/// placed this frame.
struct CloudBuffer {
    host: HostBuffer,
    vertex_count: u32,
    /// The view-projection, the offset and fog end, and the colour.
    push: [f32; 24],
    visible: bool,
}

/// One mesh's buffers: the opaque indices, then the translucent ones.
struct SectionBuffers {
    vertices: ArenaSlice,
    indices: ArenaSlice,
    index_count: u32,
    translucent_index_count: u32,
    /// Where the mesh sits, for ordering its translucent faces.
    centre: [f32; 3],
    /// When the section first showed and how long it takes to fade in.
    fade: SectionFade,
}

/// A section's fade in from the fog colour, as the reference's section
/// visibility: the time since its first mesh over the fade's length.
#[derive(Debug, Clone, Copy)]
pub struct SectionFade {
    pub since: std::time::Instant,
    pub length: std::time::Duration,
}

impl SectionFade {
    /// Shown at once.
    pub fn none() -> Self {
        Self {
            since: std::time::Instant::now(),
            length: std::time::Duration::ZERO,
        }
    }

    fn visibility(&self, now: std::time::Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.since).as_millis();
        let length = self.length.as_millis();
        if elapsed >= length {
            1.0
        } else {
            elapsed as f32 / length as f32
        }
    }
}

/// One large buffer in the GPU's memory that section meshes are carved
/// from: a few big allocations instead of two per section, which drivers
/// cap, and nothing read across the bus while drawing.
struct ArenaBlock {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,
    /// Free ranges, offset and length, in offset order. A block given back
    /// to the driver has a null buffer and none.
    free: Vec<(u64, u64)>,
}

/// A range of one arena block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArenaSlice {
    block: usize,
    offset: u64,
    size: u64,
}

/// The size of each arena block.
const ARENA_BLOCK_SIZE: u64 = 32 << 20;

/// Host memory that section bytes pass through on their way into the
/// arena, which lives where only the GPU can write.
struct StagingChunk {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapped: *mut u8,
    size: u64,
    used: u64,
}

/// One staged copy into the arena, recorded before the next frame draws.
struct StagedCopy {
    chunk: usize,
    source: u64,
    target: ArenaSlice,
    size: u64,
}

/// The size of each staging chunk.
const STAGING_CHUNK_SIZE: u64 = 4 << 20;

/// How many idle staging chunks are kept for the next frame's uploads.
const STAGING_SPARE: usize = 2;

/// How many frames the CPU may build while the GPU draws the one before.
const FRAMES_IN_FLIGHT: usize = 2;

/// What one frame in flight owns: its command buffer, the fence that says the GPU is done with it,
/// and the semaphore the swapchain image's acquisition signals.
struct FrameSlot {
    command_buffer: vk::CommandBuffer,
    in_flight: vk::Fence,
    image_available: vk::Semaphore,
    /// The number of the frame last submitted from this slot, 0 for none.
    stamp: u64,
}

/// A vertex buffer the CPU rewrites each frame: two, written alternately, so that a write never
/// waits for the frame still drawing the other. A frame that is not given new data draws the last
/// data it was given.
#[derive(Default)]
struct DynamicBuffer {
    buffers: [Option<HostBuffer>; FRAMES_IN_FLIGHT],
    /// The one the frames draw.
    active: usize,
    /// For each buffer, how many frames had been submitted when it stopped being the active one:
    /// the frames up to that number may still be drawing it.
    retired_at: [u64; FRAMES_IN_FLIGHT],
}

impl DynamicBuffer {
    /// The buffer the next frame draws.
    fn as_ref(&self) -> Option<&HostBuffer> {
        self.buffers[self.active].as_ref()
    }

    /// Free both buffers.
    fn clear(&mut self) {
        self.buffers = [None, None];
    }
}

/// A vertex buffer in host-visible, coherent memory, rewritten by the CPU every frame. It frees
/// itself when dropped.
///
/// Its owner drops it only when the GPU is done with it: after a wait for the frame in flight
/// (every replacement makes one), or once the device is idle (the backend's `Drop`, which clears
/// every owner before the device goes). It must not outlive the device.
struct HostBuffer {
    device: ash::Device,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    capacity: usize,
}

impl Drop for HostBuffer {
    fn drop(&mut self) {
        // SAFETY: this value owns both handles and runs once, so each is destroyed once; the
        // owner keeps the contract above, so the device is live and the GPU is done with them.
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

/// One frame of sky, waiting for the next render.
struct PendingSky {
    vertex_count: u32,
    batches: Vec<(crate::sky::SkyBlend, u32, u32)>,
    view_projection: [[f32; 4]; 4],
    fog_colour: [f32; 4],
    sky_end: f32,
}

/// One frame read back from the swapchain, as tightly packed RGBA8 rows.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Row-major RGBA8 samples.
    pub rgba: Vec<u8>,
}

/// A live Vulkan instance, device and swapchain for one window surface.
pub struct VulkanBackend {
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_instance: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    swapchain_device: ash::khr::swapchain::Device,
    // The instance targets Vulkan 1.2, where dynamic rendering is still an
    // extension, so its entry points come from the KHR loader rather than core.
    dynamic_rendering: ash::khr::dynamic_rendering::Device,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    command_pool: vk::CommandPool,
    /// The command buffer of the frame being recorded: one of `frames`'.
    command_buffer: vk::CommandBuffer,
    /// What each frame in flight owns.
    frames: Vec<FrameSlot>,
    /// The slot of the frame being recorded.
    frame_slot: usize,
    /// How many frames have been submitted; the next one is numbered one more.
    frames_submitted: u64,
    /// The frames up to this number are known to have finished on the GPU.
    completed: std::cell::Cell<u64>,
    /// The buffer a screenshot is copied into, there only while one is taken.
    readback: Option<(vk::Buffer, vk::DeviceMemory)>,
    depth: Option<DepthTarget>,
    /// Set when the surface no longer matches the swapchain.
    out_of_date: bool,
    scene: Option<SceneResources>,
    sky_buffer: DynamicBuffer,
    pending_sky: Option<PendingSky>,
    cloud_buffer: Option<CloudBuffer>,
    weather_buffer: DynamicBuffer,
    /// The rain's and the snow's vertex counts in the weather buffer.
    weather_counts: (u32, u32),
    /// The entities' triangles for the next frame.
    entity_buffer: DynamicBuffer,
    entity_count: u32,
    /// The dropped items' triangles, which sample the block atlas.
    ground_buffer: DynamicBuffer,
    ground_count: u32,
    /// The name tags' text: seen through blocks, then lit and depth-tested.
    name_tag_see_buffer: DynamicBuffer,
    name_tag_see_count: u32,
    /// The experience orbs' triangles, which sample the entity textures.
    orb_buffer: DynamicBuffer,
    orb_count: u32,
    /// The entity shadows' triangles, which sample the entity textures.
    shadow_buffer: DynamicBuffer,
    shadow_count: u32,
    /// The glowing layers' triangles (`RenderTypes.eyes`): the entity
    /// textures, unlit and blended, tested but not written to the depth.
    glow_buffer: DynamicBuffer,
    glow_count: u32,
    /// The energy swirls' triangles (`RenderTypes.energySwirl`).
    swirl_buffer: DynamicBuffer,
    swirl_count: u32,
    /// The enchantment glints' triangles (`RenderTypes.armorEntityGlint`).
    glint_buffer: DynamicBuffer,
    glint_count: u32,
    /// The maps' quads, drawn with the text pipeline over the maps' image.
    map_buffer: DynamicBuffer,
    map_count: u32,
    /// The decorations' sprites on them, blended over the entity textures.
    map_decoration_buffer: DynamicBuffer,
    map_decoration_count: u32,
    /// What the dissolving entities draw (a dying dragon's body), and where
    /// each texel's mask is from it in the packed image (in texture units).
    dissolve_buffer: DynamicBuffer,
    dissolve_count: u32,
    dissolve_mask_offset: [f32; 2],
    /// The dying dragon's rays.
    rays_buffer: DynamicBuffer,
    rays_count: u32,
    name_tag_buffer: DynamicBuffer,
    name_tag_count: u32,
    /// The first-person hand's triangles, in view space, and the projection
    /// they are drawn with over a cleared depth.
    hand_buffer: DynamicBuffer,
    hand_count: u32,
    /// The held items' triangles, after the arm's in the hand buffer.
    held_count: u32,
    /// The aimed block's outline strips for the next frame.
    outline_buffer: DynamicBuffer,
    outline_count: u32,
    /// The breaking overlay's triangles for the next frame.
    breaking_buffer: DynamicBuffer,
    breaking_count: u32,
    hand_projection: [[f32; 4]; 4],
    /// The GUI's triangles, their runs by blend, and the window's size in
    /// GUI units.
    gui_buffer: DynamicBuffer,
    gui_batches: Vec<(crate::gui::GuiBlend, u32, u32)>,
    gui_size: [f32; 2],
    /// The items in the GUI's slots, which the GUI's item runs count in.
    gui_item_buffer: DynamicBuffer,
    sections: std::collections::BTreeMap<(i32, i32, i32), SectionBuffers>,
    /// The sections in view this frame; `None` draws every section.
    visible_sections: Option<Vec<(i32, i32, i32)>>,
    /// The falling blocks' meshes for the frame, drawn with the sections.
    moving_sections: Vec<SectionBuffers>,
    arena: Vec<ArenaBlock>,
    /// Staging chunks: every chunk, the ones taking uploads for the next
    /// frame, and the ones the frame in flight still copies from.
    staging: Vec<StagingChunk>,
    staging_filling: Vec<usize>,
    staging_in_flight: Vec<(u64, usize)>,
    staged: Vec<StagedCopy>,
    /// Section meshes replaced since the frame in flight began; freed once
    /// it is done with them.
    retired: Vec<(u64, SectionBuffers)>,
    sort_origin: [f32; 3],
    lightmap: [f32; 12],
    /// The terrain fog's colour and its four distances.
    fog: [f32; 8],
    render_finished: Vec<vk::Semaphore>,
    /// The validation layer's messenger, with `VOXEL_VALIDATION`.
    validation: Option<validation::Validation>,
    /// The pipeline cache, null until [`VulkanBackend::use_pipeline_cache`], and its file.
    pipeline_cache: vk::PipelineCache,
    pipeline_cache_path: Option<std::path::PathBuf>,
    cache_identity: pipeline_cache::CacheIdentity,
    /// GPU timestamp queries, when the device and queue can write them.
    timestamps: Option<TimestampQueries>,
    /// The CPU time of the last frame, by span.
    timings: BackendTimings,
    /// The time spent waiting for the frame in flight since the last frame was submitted.
    fence_wait: std::cell::Cell<std::time::Duration>,
    description: BackendDescription,
}
