use super::*;

/// No terrain fog: every distance out of reach, as the reference's empty
/// fog buffer.
pub(super) const NO_FOG: [f32; 8] = [0.0, 0.0, 0.0, 1.0, f32::MAX, f32::MAX, f32::MAX, f32::MAX];

/// Lightmap inputs with nothing dimming them: white sky light at full
/// factor, the default block light tint at the reference's base factor, no
/// ambient light, the default brightness option, no darkness effect and no boss bars.
pub(super) const DEFAULT_LIGHTMAP: [f32; 15] = [
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
    0.0,
    0.0,
    0.0,
];

/// Which of the drawing pipelines to build.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PipelineKind {
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
    /// A text selection, added to what is behind it (`GUI_TEXT_HIGHLIGHT`).
    GuiHighlight,
    GuiVignette,
    /// Items in GUI slots, over a cleared depth.
    GuiItem,
    /// The glint over an item in a GUI slot, over the depth the item wrote.
    GuiGlint,
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
    /// The world border overlay (`WORLD_BORDER`): the forcefield texture, tinted and blended by
    /// `BlendFunction.OVERLAY`, pulled towards the camera by a depth bias.
    WorldBorder,
    /// A screen effect (`BLOCK_SCREEN_EFFECT`, `FIRE_SCREEN_EFFECT`): the hand's shaders, blended
    /// and tested against nothing.
    ScreenEffect,
    /// The end portal and end gateway surfaces (`END_PORTAL`, `END_GATEWAY`): the end sky and the
    /// portal's picture layered over where the surface is on the screen, opaque. Two sets are
    /// bound, one a texture each.
    Portal,
}

/// Instance API version the reference asks for.
pub(super) const API_VERSION: u32 = vk::API_VERSION_1_2;

/// Lowest device API version the reference's feature query accepts.
pub(super) const MINIMUM_DEVICE_API_VERSION: u32 = vk::API_VERSION_1_1;

/// No transform, for the frame paths that draw nothing.
pub(super) const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Swapchain images the reference requests, clamped up by surface capabilities.
pub(super) const REQUESTED_IMAGE_COUNT: u32 = 3;

/// Device extensions the reference requires before a GPU is considered usable.
pub(super) const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 5] = [
    c"VK_KHR_dynamic_rendering",
    c"VK_KHR_push_descriptor",
    c"VK_KHR_synchronization2",
    c"VK_EXT_vertex_attribute_divisor",
    c"VK_KHR_swapchain",
];

/// Surface formats the reference accepts, paired with an sRGB non-linear space.
pub(super) const ACCEPTED_SURFACE_FORMATS: [vk::Format; 2] =
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
    /// `VkPhysicalDeviceProperties::vendorID`.
    pub vendor_id: u32,
    /// `VkPhysicalDeviceProperties::deviceType`: 1 integrated, 2 discrete, 3 virtual, 4 CPU.
    pub device_type: i32,
    /// `VulkanPhysicalDevice.driverInfo`: the API version, the driver's name and its info, such as
    /// `1.4.311 NVIDIA 570.133.07`.
    pub driver_info: String,
    /// Chosen swapchain image format.
    pub surface_format: vk::Format,
    /// Number of swapchain images actually created.
    pub image_count: u32,
    /// Swapchain extent in pixels.
    pub extent: vk::Extent2D,
    /// How the swapchain presents: FIFO waits for the display, IMMEDIATE does not.
    pub present_mode: vk::PresentModeKHR,
}

/// The query pool the scene's pass timestamps are written to, and what turns
/// its ticks into time.
pub(super) struct TimestampQueries {
    pub(super) pool: vk::QueryPool,
    /// Nanoseconds one tick lasts.
    pub(super) period_ns: f64,
    /// How many bits of a tick the queue writes.
    pub(super) valid_bits: u32,
    /// Whether each slot's last frame wrote a scene's timestamps not yet read.
    pub(super) pending: [bool; FRAMES_IN_FLIGHT],
    /// The last frame read back.
    pub(super) last: Option<GpuTimings>,
}

/// The pipelines the GUI draws its special blends with.
pub(super) struct GuiPipelines {
    pub(super) invert: Pipeline,
    pub(super) highlight: Pipeline,
    pub(super) vignette: Pipeline,
    pub(super) item: Pipeline,
    pub(super) glint: Pipeline,
}

/// Everything needed to draw one textured mesh.
pub(super) struct SceneResources {
    pub(super) texture: Texture,
    pub(super) descriptors: Descriptors,
    pub(super) descriptor_set: vk::DescriptorSet,
    pub(super) pipeline: Pipeline,
    pub(super) translucent_pipeline: Pipeline,
    pub(super) sky_pipeline: Pipeline,
    pub(super) sky_overlay_pipeline: Pipeline,
    pub(super) cloud_pipeline: Pipeline,
    pub(super) weather_pipeline: Pipeline,
    /// The rain and snow textures, once loaded, with their sampler and sets.
    pub(super) weather_textures: Vec<Image>,
    pub(super) weather_sampler: Option<Sampler>,
    pub(super) weather_sets: Vec<vk::DescriptorSet>,
    pub(super) entity_pipeline: Pipeline,
    pub(super) lines_pipeline: Pipeline,
    pub(super) crumbling_pipeline: Pipeline,
    pub(super) held_pipeline: Pipeline,
    pub(super) text_pipeline: Pipeline,
    pub(super) text_see_through_pipeline: Pipeline,
    pub(super) text_translucent_pipeline: Pipeline,
    pub(super) shadow_pipeline: Pipeline,
    pub(super) swirl_pipeline: Pipeline,
    pub(super) glint_pipeline: Pipeline,
    pub(super) dissolve_pipeline: Pipeline,
    pub(super) rays_pipeline: Pipeline,
    pub(super) world_border_pipeline: Pipeline,
    /// The forcefield texture, once loaded, with its set.
    pub(super) world_border_texture: Option<Texture>,
    pub(super) world_border_set: Option<vk::DescriptorSet>,
    pub(super) screen_effect_pipeline: Pipeline,
    /// The water texture laid over the view under water, once loaded, with its set.
    pub(super) underwater_texture: Option<Texture>,
    pub(super) underwater_set: Option<vk::DescriptorSet>,
    pub(super) crack_texture: Option<Texture>,
    pub(super) crack_set: Option<vk::DescriptorSet>,
    /// The skin entities are drawn with.
    pub(super) entity_texture: Option<Texture>,
    pub(super) entity_set: Option<vk::DescriptorSet>,
    pub(super) gui_pipeline: Pipeline,
    pub(super) gui_pipelines: GuiPipelines,
    /// The GUI sprites' atlas.
    pub(super) gui_texture: Option<Texture>,
    pub(super) gui_set: Option<vk::DescriptorSet>,
    /// The same atlas sampled smoothly, for the textures whose metadata asks for it (`blur`).
    pub(super) gui_blur: Option<(vk::DescriptorSet, Sampler)>,
    /// The maps' colours and the decoration sprites, packed in one image, and its sampler and set.
    pub(super) map_texture: Option<Texture>,
    pub(super) map_set: Option<vk::DescriptorSet>,
    /// The title screen's cube map, once loaded.
    pub(super) panorama: Option<super::panorama::PanoramaScene>,
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
pub(super) struct CloudBuffer {
    pub(super) host: HostBuffer,
    pub(super) vertex_count: u32,
    /// The view-projection, the offset and fog end, and the colour.
    pub(super) push: [f32; 24],
    pub(super) visible: bool,
}

/// What the arena gives back once the frames that may read it are done.
pub(super) enum Retired {
    /// A section mesh that was replaced.
    Section(SectionBuffers),
    /// The index range a section left when its translucent faces were
    /// ordered again.
    Indices(ArenaSlice),
}

/// One mesh's buffers: the opaque indices, then the translucent ones.
pub(super) struct SectionBuffers {
    pub(super) vertices: ArenaSlice,
    pub(super) indices: ArenaSlice,
    pub(super) index_count: u32,
    pub(super) translucent_index_count: u32,
    /// Where the mesh sits, for ordering its translucent faces.
    pub(super) centre: [f32; 3],
    /// When the section first showed and how long it takes to fade in.
    pub(super) fade: SectionFade,
    /// What ordering its translucent faces again needs.
    pub(super) sort: Option<Box<TranslucentSort>>,
}

/// A section's translucent faces kept to be ordered again as the camera
/// moves (`MeshData.SortState`).
pub(super) struct TranslucentSort {
    /// The indices as uploaded: the opaque ones, then the translucent.
    pub(super) indices: Vec<u32>,
    /// How many of them are opaque.
    pub(super) opaque: usize,
    /// The translucent faces' indices as the mesh gave them, which the
    /// centroids follow.
    pub(super) faces: Vec<[u32; 6]>,
    /// The middle of each translucent face, which sits on one or two
    /// sextets of indices.
    pub(super) centroids: Vec<[f32; 3]>,
    /// Where the camera stood among the section's neighbours when it was
    /// last ordered (`TranslucencyPointOfView`).
    pub(super) point_of_view: [i32; 3],
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

    pub fn visibility(&self, now: std::time::Instant) -> f32 {
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
pub(super) struct ArenaBlock {
    pub(super) buffer: BoundBuffer,
    pub(super) size: u64,
    /// Free ranges, offset and length, in offset order.
    pub(super) free: Vec<(u64, u64)>,
}

/// What the section meshes hold.
#[derive(Debug, Clone, Copy, Default)]
pub struct SectionMemory {
    pub sections: usize,
    /// Bytes of the sections' vertices and of their indices, as the arena rounds them.
    pub vertex_bytes: u64,
    pub index_bytes: u64,
    /// Bytes of the arena blocks that carry them, free ranges included.
    pub arena_bytes: u64,
}

/// A range of one arena block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ArenaSlice {
    pub(super) block: usize,
    pub(super) offset: u64,
    pub(super) size: u64,
}

/// The size of each arena block.
pub(super) const ARENA_BLOCK_SIZE: u64 = 32 << 20;

/// Host memory that section bytes pass through on their way into the
/// arena, which lives where only the GPU can write.
pub(super) struct StagingChunk {
    pub(super) buffer: BoundBuffer,
    pub(super) mapped: *mut u8,
    pub(super) size: u64,
    pub(super) used: u64,
}

impl Drop for StagingChunk {
    fn drop(&mut self) {
        // SAFETY: the memory is mapped once, for good, when the chunk is made, and this runs once,
        // before the buffer and its memory are freed with the field.
        unsafe { self.buffer.device.unmap_memory(self.buffer.memory) };
    }
}

/// One staged copy into the arena, recorded before the next frame draws.
pub(super) struct StagedCopy {
    pub(super) chunk: usize,
    pub(super) source: u64,
    pub(super) target: ArenaSlice,
    pub(super) size: u64,
}

/// The size of each staging chunk.
pub(super) const STAGING_CHUNK_SIZE: u64 = 4 << 20;

/// How many idle staging chunks are kept for the next frame's uploads.
pub(super) const STAGING_SPARE: usize = 2;

/// How many frames the CPU may build while the GPU draws the one before.
pub(super) const FRAMES_IN_FLIGHT: usize = 2;

/// What one frame in flight owns: its command buffer, the fence that says the GPU is done with it,
/// and the semaphore the swapchain image's acquisition signals.
pub(super) struct FrameSlot {
    pub(super) command_buffer: vk::CommandBuffer,
    pub(super) in_flight: vk::Fence,
    pub(super) image_available: vk::Semaphore,
    /// The number of the frame last submitted from this slot, 0 for none.
    pub(super) stamp: u64,
}

/// A vertex buffer the CPU rewrites each frame: two, written alternately, so that a write never
/// waits for the frame still drawing the other. A frame that is not given new data draws the last
/// data it was given.
#[derive(Default)]
pub(super) struct DynamicBuffer {
    pub(super) buffers: [Option<HostBuffer>; FRAMES_IN_FLIGHT],
    /// The one the frames draw.
    pub(super) active: usize,
    /// For each buffer, how many frames had been submitted when it stopped being the active one:
    /// the frames up to that number may still be drawing it.
    pub(super) retired_at: [u64; FRAMES_IN_FLIGHT],
}

impl DynamicBuffer {
    /// The buffer the next frame draws.
    pub(super) fn as_ref(&self) -> Option<&HostBuffer> {
        self.buffers[self.active].as_ref()
    }

    /// Free both buffers.
    pub(super) fn clear(&mut self) {
        self.buffers = [None, None];
    }
}

/// A vertex buffer in host-visible, coherent memory, rewritten by the CPU every frame. It frees
/// itself when dropped.
///
/// Its owner drops it only when the GPU is done with it: after a wait for the frame in flight
/// (every replacement makes one), or once the device is idle (the backend's `Drop`, which clears
/// every owner before the device goes). It must not outlive the device.
pub(super) struct HostBuffer {
    pub(super) bound: BoundBuffer,
    /// The whole buffer, mapped once when it is made: mapping again each frame costs the driver
    /// a fresh mapping and the CPU its page faults.
    pub(super) mapped: *mut u8,
    pub(super) capacity: usize,
}

impl Drop for HostBuffer {
    fn drop(&mut self) {
        // SAFETY: the memory is mapped once, for good, when the buffer is made, and this runs
        // once, before the buffer and its memory are freed with the field.
        unsafe { self.bound.device.unmap_memory(self.bound.memory) };
    }
}

impl std::ops::Deref for HostBuffer {
    type Target = BoundBuffer;

    fn deref(&self) -> &BoundBuffer {
        &self.bound
    }
}

/// One frame of sky, waiting for the next render.
pub(super) struct PendingSky {
    pub(super) vertex_count: u32,
    pub(super) batches: Vec<(crate::sky::SkyBlend, u32, u32)>,
    pub(super) view_projection: [[f32; 4]; 4],
    pub(super) fog_colour: [f32; 4],
    pub(super) sky_end: f32,
}

/// What one kind of draw keeps between frames: the vertices it draws and how many of them there
/// are. A frame that is not given new vertices draws the last ones.
#[derive(Default)]
pub(super) struct DrawList {
    pub(super) buffer: DynamicBuffer,
    pub(super) count: u32,
}
