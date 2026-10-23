use super::*;

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
    pub(super) _entry: ash::Entry,
    pub(super) instance: ash::Instance,
    pub(super) surface_instance: ash::khr::surface::Instance,
    pub(super) surface: vk::SurfaceKHR,
    pub(super) physical_device: vk::PhysicalDevice,
    pub(super) device: ash::Device,
    pub(super) queue: vk::Queue,
    pub(super) swapchain_device: ash::khr::swapchain::Device,
    // The instance targets Vulkan 1.2, where dynamic rendering is still an
    // extension, so its entry points come from the KHR loader rather than core.
    pub(super) dynamic_rendering: ash::khr::dynamic_rendering::Device,
    pub(super) swapchain: vk::SwapchainKHR,
    pub(super) images: Vec<vk::Image>,
    pub(super) image_views: Vec<vk::ImageView>,
    pub(super) command_pool: vk::CommandPool,
    /// The command buffer of the frame being recorded: one of `frames`'.
    pub(super) command_buffer: vk::CommandBuffer,
    /// What each frame in flight owns.
    pub(super) frames: Vec<FrameSlot>,
    /// The slot of the frame being recorded.
    pub(super) frame_slot: usize,
    /// How many frames have been submitted; the next one is numbered one more.
    pub(super) frames_submitted: u64,
    /// The frames up to this number are known to have finished on the GPU.
    pub(super) completed: std::cell::Cell<u64>,
    /// The buffer a screenshot is copied into, there only while one is taken.
    pub(super) readback: Option<BoundBuffer>,
    pub(super) depth: Option<Image>,
    /// Set when the surface no longer matches the swapchain.
    pub(super) out_of_date: bool,
    /// Draw only the GUI: a menu with no level behind it.
    pub(super) gui_only: bool,
    /// How the terrain samples the atlas (`textureFiltering`).
    pub(super) filtering: super::terrain_filter::TerrainFiltering,
    pub(super) scene: Option<SceneResources>,
    pub(super) sky_buffer: DynamicBuffer,
    pub(super) pending_sky: Option<PendingSky>,
    /// The title screen's cube map: where it is seen from, column by column; none to draw none.
    pub(super) panorama_view: Option<[f32; 16]>,
    /// How far the cube is blurred, in texels; 0 for not at all.
    pub(super) panorama_blur: u32,
    pub(super) cloud_buffer: Option<CloudBuffer>,
    pub(super) weather_buffer: DynamicBuffer,
    /// The rain's and the snow's vertex counts in the weather buffer.
    pub(super) weather_counts: (u32, u32),
    /// The world border overlay's triangles for the next frame.
    pub(super) world_border_buffer: DynamicBuffer,
    pub(super) world_border_count: u32,
    /// The screen effects' triangles for the next frame: the block the eyes are in, the water, the
    /// fire, and how many vertices each has.
    pub(super) screen_effect_buffer: DynamicBuffer,
    pub(super) screen_effect_counts: [u32; 3],
    pub(super) screen_effect_projection: [[f32; 4]; 4],
    /// The entities' triangles for the next frame.
    pub(super) entity: DrawList,
    /// The dropped items' triangles, which sample the block atlas: the opaque ones, then the quads
    /// of translucent sprites.
    pub(super) ground: DrawList,
    /// The name tags' text: seen through blocks, then lit and depth-tested.
    pub(super) name_tag_see: DrawList,
    /// The experience orbs' triangles, which sample the entity textures.
    pub(super) orb: DrawList,
    /// The particles' triangles, which sample the entity textures.
    pub(super) particle: DrawList,
    /// The flakes of blocks' triangles, which sample the block atlas.
    pub(super) block_particle: DrawList,
    /// The entity shadows' triangles, which sample the entity textures.
    pub(super) shadow: DrawList,
    /// The glowing layers' triangles (`RenderTypes.eyes`): the entity
    /// textures, unlit and blended, tested but not written to the depth.
    pub(super) glow: DrawList,
    /// The energy swirls' triangles (`RenderTypes.energySwirl`).
    pub(super) swirl: DrawList,
    /// The enchantment glints' triangles (`RenderTypes.armorEntityGlint`).
    pub(super) glint: DrawList,
    /// The maps' quads, drawn with the text pipeline over the maps' image.
    pub(super) map: DrawList,
    /// The decorations' sprites on them, blended over the entity textures.
    pub(super) map_decoration: DrawList,
    /// What the dissolving entities draw (a dying dragon's body), and where
    /// each texel's mask is from it in the packed image (in texture units).
    pub(super) dissolve: DrawList,
    pub(super) dissolve_mask_offset: [f32; 2],
    /// The dying dragon's rays.
    pub(super) rays: DrawList,
    /// The end portal and gateway surfaces, and how far through the day the game is.
    pub(super) portal: DrawList,
    /// The portal's pipeline and the two textures it layers, once loaded.
    pub(super) portal_scene: Option<super::portal::PortalScene>,
    pub(super) portal_time: f32,
    pub(super) name_tag: DrawList,
    /// The first-person hand's triangles, in view space, and the projection
    /// they are drawn with over a cleared depth.
    pub(super) hand: DrawList,
    /// The held items' triangles and then their glints' (counts), after the arm's in the hand
    /// buffer.
    pub(super) held_counts: [u32; 3],
    /// The aimed block's outline strips and the F3 3D crosshair for the next frame.
    pub(super) lines: LineDraws,
    /// `glintStrength`: how bright the enchantment glints are drawn.
    pub(super) glint_strength: f32,
    /// The breaking overlay's triangles for the next frame.
    pub(super) breaking: DrawList,
    pub(super) hand_projection: [[f32; 4]; 4],
    /// The GUI's triangles, their runs by blend, and the window's size in
    /// GUI units.
    pub(super) gui_buffer: DynamicBuffer,
    pub(super) gui_runs: GuiRuns,
    pub(super) gui_size: [f32; 2],
    /// The items in the GUI's slots, which the GUI's item runs count in.
    pub(super) gui_item_buffer: DynamicBuffer,
    pub(super) sections: std::collections::BTreeMap<(i32, i32, i32), SectionBuffers>,
    /// The sections in view this frame; `None` draws every section.
    pub(super) visible_sections: Option<Vec<(i32, i32, i32)>>,
    /// The falling blocks' meshes for the frame, drawn with the sections.
    pub(super) moving_sections: Vec<SectionBuffers>,
    pub(super) arena: Vec<Option<ArenaBlock>>,
    /// Staging chunks: every chunk, the ones taking uploads for the next
    /// frame, and the ones the frame in flight still copies from.
    pub(super) staging: Vec<Option<StagingChunk>>,
    pub(super) staging_filling: Vec<usize>,
    pub(super) staging_in_flight: Vec<(u64, usize)>,
    pub(super) staged: Vec<StagedCopy>,
    /// Section meshes replaced since the frame in flight began; freed once
    /// it is done with them.
    pub(super) retired: Vec<(u64, Retired)>,
    pub(super) sort_origin: [f32; 3],
    /// The lightmap's inputs, then the darkness effect's subtraction, which the push constants
    /// carry in the camera's `w`, and the boss bars' darkening of the world.
    pub(super) lightmap: [f32; 15],
    /// The terrain fog's colour and its four distances.
    pub(super) fog: [f32; 8],
    pub(super) render_finished: Vec<vk::Semaphore>,
    /// Whether frames wait for the display; the swapchain's present mode follows.
    pub(super) vsync: bool,
    /// The validation layer's messenger, with `PHOTON_VALIDATION`.
    pub(super) validation: Option<validation::Validation>,
    /// The pipeline cache, null until [`VulkanBackend::use_pipeline_cache`], and its file.
    pub(super) pipeline_cache: vk::PipelineCache,
    pub(super) pipeline_cache_path: Option<std::path::PathBuf>,
    pub(super) cache_identity: pipeline_cache::CacheIdentity,
    /// GPU timestamp queries, when the device and queue can write them.
    pub(super) timestamps: Option<TimestampQueries>,
    /// The CPU time of the last frame, by span.
    pub(super) timings: BackendTimings,
    /// The time spent waiting for the frame in flight since the last frame was submitted.
    pub(super) fence_wait: std::cell::Cell<std::time::Duration>,
    pub(super) description: BackendDescription,
}

/// The GUI's runs by blend, and where the frame behind the screen is blurred
/// (`GuiRenderState.firstStratumAfterBlur`).
#[derive(Default)]
pub(super) struct GuiRuns {
    pub(super) batches: Vec<(crate::gui::GuiBlend, u32, u32)>,
    pub(super) blur: Option<crate::gui::frame::GuiBlur>,
}
