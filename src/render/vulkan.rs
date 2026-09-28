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

use std::ffi::{CStr, CString};

use ash::vk;

use crate::render::png::DecodedImage;
use crate::render::scene::{Mesh, Vertex};

/// Scaffolding shaders. These are this project's own, not the reference's:
/// they exist to prove the pipeline works, and carry no parity claim.
/// What the terrain's and the entities' vertex shaders share: the push
/// constants, the vertex inputs and the lightmap.
macro_rules! vertex_prelude {
    () => {
        r"#version 450
layout(push_constant) uniform Push {
    mat4 view_projection;
    vec4 sky_colour_factor;
    vec4 block_tint_factor;
    vec4 ambient_brightness;
    vec4 camera;
    vec4 fog_colour;
    vec4 fog_distances;
} push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in float in_shade;
layout(location = 3) in vec4 in_tint;
layout(location = 4) in vec2 in_light;

// One texel of the reference's 16x16 lightmap, as its lightmap shader
// computes it and as an 8-bit texture stores it.
float brightness(float level) {
    return level / (4.0 - 3.0 * level);
}
vec3 lightmap_texel(int block, int sky) {
    float block_level = float(block) / 15.0;
    float sky_level = float(sky) / 15.0;
    float block_brightness = brightness(block_level) * push.block_tint_factor.a;
    float sky_brightness = brightness(sky_level) * push.sky_colour_factor.a;
    vec3 colour = push.ambient_brightness.rgb;
    colour += push.sky_colour_factor.rgb * sky_brightness;
    float parabola = (2.0 * block_level - 1.0) * (2.0 * block_level - 1.0);
    colour += mix(push.block_tint_factor.rgb, vec3(1.0), 0.9 * parabola) * block_brightness;
    colour = clamp(colour, 0.0, 1.0);
    float top = max(max(colour.x, colour.y), colour.z);
    float inverted = 1.0 - top;
    vec3 lifted = colour * ((1.0 - inverted * inverted * inverted * inverted) / top);
    colour = mix(colour, lifted, push.ambient_brightness.a);
    return floor(colour * 255.0 + 0.5) / 255.0;
}
// The lightmap sampled with linear filtering at the vertex's coordinates.
vec3 sample_lightmap(vec2 coords) {
    vec2 uv = clamp(coords / 256.0 + 0.5 / 16.0, vec2(0.5 / 16.0), vec2(15.5 / 16.0));
    vec2 texel = uv * 16.0 - 0.5;
    ivec2 low = ivec2(floor(texel));
    vec2 fraction = texel - floor(texel);
    ivec2 high = min(low + 1, ivec2(15));
    vec3 bottom = mix(lightmap_texel(low.x, low.y), lightmap_texel(high.x, low.y), fraction.x);
    vec3 top = mix(lightmap_texel(low.x, high.y), lightmap_texel(high.x, high.y), fraction.x);
    return mix(bottom, top, fraction.y);
}
"
    };
}

const VERTEX_SHADER: &str = concat!(
    vertex_prelude!(),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out float out_shade;
layout(location = 2) out vec4 out_tint;
layout(location = 3) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_shade = in_shade;
    out_tint = in_tint * vec4(sample_lightmap(in_light), 1.0);
    // Fog is measured from the camera: once as a sphere, once as a
    // cylinder whose height counts like its radius.
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The entities' vertex shader, after the reference's entity shader with
/// per-face lighting: the vertex's normal rides in its tint, and both faces'
/// light is worked out so the fragment can pick by which way it faces.
const ENTITY_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec3 out_front;
layout(location = 2) out vec3 out_back;
layout(location = 3) out vec3 out_light;
layout(location = 4) out vec2 out_fog;
// The level's two diffuse lights.
const vec3 LIGHT0 = normalize(vec3(0.2, 1.0, -0.7));
const vec3 LIGHT1 = normalize(vec3(-0.2, 1.0, 0.7));
float mix_light(vec2 light) {
    vec2 value = max(vec2(0.0), light);
    return min(1.0, (value.x + value.y) * 0.6 + 0.4);
}
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    vec2 light = vec2(dot(LIGHT0, in_tint.xyz), dot(LIGHT1, in_tint.xyz));
    out_front = vec3(mix_light(light));
    out_back = vec3(mix_light(-light));
    out_light = sample_lightmap(in_light);
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The entities' fragment shader: the texture, cut out below a tenth of
/// alpha, lit by whichever face shows, the lightmap, then fog.
const ENTITY_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec3 in_front;
layout(location = 2) in vec3 in_back;
layout(location = 3) in vec3 in_light;
layout(location = 4) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
void main() {
    vec4 colour = texture(skin, in_uv);
    if (colour.a < 0.1) { discard; }
    colour.rgb *= gl_FrontFacing ? in_front : in_back;
    colour.rgb *= in_light;
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

const FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in float in_shade;
layout(location = 2) in vec4 in_tint;
layout(location = 3) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
    // How far the section has faded in from the fog colour.
    float visibility;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
// The terrain sampler filters linearly, yet the reference draws blocks
// pixel-sharp: its terrain shader pulls each lookup towards the centre of the
// texel it lands in, by as much as that texel is larger than a screen pixel.
// Up close the lookup sits on the texel centre and reads it alone; only in the
// last screen pixel of a texel edge does it blend with the neighbour, which is
// what keeps the edges from shimmering. Far away, where a texel is smaller
// than a pixel, the lookup is left alone and the mip chain does the work.
vec4 sample_texel_centred(vec2 uv) {
    vec2 texel_size = 1.0 / vec2(textureSize(atlas, 0));
    vec2 du = dFdx(uv);
    vec2 dv = dFdy(uv);
    vec2 texel_on_screen = sqrt(du * du + dv * dv);
    vec2 texel_coords = uv / texel_size;
    vec2 corner = round(texel_coords) - 0.5;
    vec2 offset = texel_coords - corner;
    offset = clamp((offset - 0.5) * texel_size / texel_on_screen + 0.5, 0.0, 1.0);
    return textureGrad(atlas, (corner + offset) * texel_size, du, dv);
}
void main() {
    vec4 texel = sample_texel_centred(in_uv) * in_tint;
    texel.rgb *= in_shade;
    texel = mix(vec4(push.fog_colour.rgb, texel.a), texel, push.visibility);
    if (texel.a < 0.1) { discard; }
    vec3 colour = texel.rgb;
    // The environment's fog over the spherical distance, the render
    // distance's over the cylindrical one, whichever is thicker.
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour, push.fog_colour.rgb, fog * push.fog_colour.a), texel.a);
}
";

/// The sky's shaders. Colour comes from the vertex, a texture only when the
/// vertex carries atlas coordinates, and fog as the reference's sky shader
/// applies it: towards the fog colour over the spherical distance, and fully
/// past the same distance measured cylindrically.
const SKY_VERTEX_SHADER: &str = r"#version 450
layout(push_constant) uniform Push { mat4 view_projection; } push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_colour;
layout(location = 3) in vec2 in_fog;
layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_colour;
layout(location = 2) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_colour = in_colour;
    out_fog = in_fog;
}
";

const SKY_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(push_constant) uniform Push { layout(offset = 64) vec4 fog_colour; float sky_end; } push;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
void main() {
    vec4 colour = in_colour;
    if (in_uv.x >= 0.0) {
        // Celestial sprites are sampled at the nearest texel of the top level.
        ivec2 size = textureSize(atlas, 0);
        vec4 texel = texelFetch(atlas, clamp(ivec2(floor(in_uv * vec2(size))), ivec2(0), size - 1), 0);
        if (texel.a == 0.0) { discard; }
        colour *= texel;
    }
    if (in_fog.x >= 0.0) {
        float spherical = in_fog.x <= 0.0 ? 0.0 : (in_fog.x >= push.sky_end ? 1.0 : in_fog.x / push.sky_end);
        float cylindrical = in_fog.y <= push.sky_end ? 0.0 : 1.0;
        float fog = max(spherical, cylindrical);
        colour.rgb = mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a);
    }
    out_colour = colour;
}
";

/// The clouds' shaders, after the reference's: each vertex is placed from
/// the camera's cell corner plus the camera's offset in its cell, coloured by
/// its face's brightness times the cloud colour, and faded out towards the
/// cloud fog's end over its spherical distance.
const CLOUD_VERTEX_SHADER: &str = r"#version 450
layout(push_constant) uniform Push {
    mat4 view_projection;
    vec4 offset_and_end;
    vec4 colour;
} push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in float in_shade;
layout(location = 0) out vec4 out_colour;
layout(location = 1) out float out_distance;
void main() {
    vec3 position = in_position + push.offset_and_end.xyz;
    gl_Position = push.view_projection * vec4(position, 1.0);
    out_colour = vec4(vec3(in_shade), 1.0) * push.colour;
    out_distance = length(position);
}
";

const CLOUD_FRAGMENT_SHADER: &str = r"#version 450
layout(push_constant) uniform Push {
    layout(offset = 64) vec4 offset_and_end;
} push;
layout(location = 0) in vec4 in_colour;
layout(location = 1) in float in_distance;
layout(location = 0) out vec4 out_colour;
void main() {
    float end = push.offset_and_end.w;
    float fog = in_distance <= 0.0 ? 0.0 : (in_distance >= end ? 1.0 : in_distance / end);
    out_colour = vec4(in_colour.rgb, in_colour.a * (1.0 - fog));
}
";

/// The weather's fragment shader, after the reference's particle shader:
/// the texture at its plain lookup times the vertex colour, a cutout, then
/// the terrain fog. The vertex side is the terrain's.
const WEATHER_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D weather;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in float in_shade;
layout(location = 2) in vec4 in_tint;
layout(location = 3) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
    // How far the section has faded in from the fog colour.
    float visibility;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
void main() {
    vec4 colour = texture(weather, in_uv) * in_tint;
    if (colour.a < 0.1) { discard; }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

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
    /// The skin entities are drawn with.
    entity_texture: Option<(vk::Image, vk::DeviceMemory, vk::ImageView)>,
    entity_sampler: Option<vk::Sampler>,
    entity_set: Option<vk::DescriptorSet>,
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
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    capacity: usize,
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
    /// Free ranges, offset and length, in offset order.
    free: Vec<(u64, u64)>,
}

/// A range of one arena block.
#[derive(Debug, Clone, Copy)]
struct ArenaSlice {
    block: usize,
    offset: u64,
    size: u64,
}

/// The size of each arena block.
const ARENA_BLOCK_SIZE: u64 = 64 << 20;

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
const STAGING_CHUNK_SIZE: u64 = 16 << 20;

/// The sky's vertex buffer, rewritten every frame.
struct SkyBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    capacity: usize,
}

/// One frame of sky, waiting for the next render.
struct PendingSky {
    vertex_count: u32,
    batches: Vec<(crate::render::sky::SkyBlend, u32, u32)>,
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
    command_buffer: vk::CommandBuffer,
    readback_buffer: vk::Buffer,
    readback_memory: vk::DeviceMemory,
    depth: Option<DepthTarget>,
    /// Set when the surface no longer matches the swapchain.
    out_of_date: bool,
    scene: Option<SceneResources>,
    sky_buffer: Option<SkyBuffer>,
    pending_sky: Option<PendingSky>,
    cloud_buffer: Option<CloudBuffer>,
    weather_buffer: Option<SkyBuffer>,
    /// The rain's and the snow's vertex counts in the weather buffer.
    weather_counts: (u32, u32),
    /// The entities' triangles for the next frame.
    entity_buffer: Option<SkyBuffer>,
    entity_count: u32,
    sections: std::collections::BTreeMap<(i32, i32, i32), SectionBuffers>,
    arena: Vec<ArenaBlock>,
    /// Staging chunks: every chunk, the ones taking uploads for the next
    /// frame, and the ones the frame in flight still copies from.
    staging: Vec<StagingChunk>,
    staging_filling: Vec<usize>,
    staging_in_flight: Vec<usize>,
    staged: Vec<StagedCopy>,
    /// Section meshes replaced since the frame in flight began; freed once
    /// it is done with them.
    retired: Vec<SectionBuffers>,
    sort_origin: [f32; 3],
    lightmap: [f32; 12],
    /// The terrain fog's colour and its four distances.
    fog: [f32; 8],
    image_available: vk::Semaphore,
    render_finished: Vec<vk::Semaphore>,
    in_flight: vk::Fence,
    description: BackendDescription,
}

impl VulkanBackend {
    /// Bring the backend up against an existing window surface.
    ///
    /// # Safety
    ///
    /// `display_handle` and `window_handle` must refer to a window that outlives
    /// this backend; the surface borrows them.
    pub unsafe fn create(
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanError> {
        unsafe {
            let entry =
                ash::Entry::load().map_err(|error| VulkanError::Loader(error.to_string()))?;

            // The reference takes its instance extensions from GLFW and names
            // the application the same way on every run.
            let mut extensions = ash_window::enumerate_required_extensions(display_handle)
                .map_err(|result| VulkanError::Call("enumerate surface extensions", result))?
                .to_vec();
            extensions.retain(|name| !name.is_null());

            let application_name = CString::new("Minecraft Java Edition").expect("static name");
            let engine_name = CString::new("MinecraftJE").expect("static name");
            let application_info = vk::ApplicationInfo::default()
                .application_name(&application_name)
                .engine_name(&engine_name)
                .engine_version(0)
                .api_version(API_VERSION);
            let instance_info = vk::InstanceCreateInfo::default()
                .application_info(&application_info)
                .enabled_extension_names(&extensions);
            let instance = entry
                .create_instance(&instance_info, None)
                .map_err(|result| VulkanError::Call("vkCreateInstance", result))?;

            let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);
            let surface =
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                    .map_err(|result| VulkanError::Call("vkCreateSurfaceKHR", result))?;

            let (physical_device, queue_family, properties) =
                select_physical_device(&instance, &surface_instance, surface)?;

            let queue_priorities = [1.0_f32];
            let queue_info = [vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family)
                .queue_priorities(&queue_priorities)];
            let extension_names: Vec<*const i8> = REQUIRED_DEVICE_EXTENSIONS
                .iter()
                .map(|name| name.as_ptr())
                .collect();

            // The reference enables these by name; the ones that gate rendering
            // are requested here so device selection matches what it would pick.
            let mut features_11 =
                vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
            let mut features_12 = vk::PhysicalDeviceVulkan12Features::default()
                .timeline_semaphore(true)
                .host_query_reset(true);
            let mut features_sync2 =
                vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);
            let mut features_dynamic =
                vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);
            let mut features_divisor =
                vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT::default()
                    .vertex_attribute_instance_rate_divisor(true);
            let features_10 = vk::PhysicalDeviceFeatures::default()
                .multi_draw_indirect(true)
                .fill_mode_non_solid(true)
                .sampler_anisotropy(true);

            let device_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&extension_names)
                .enabled_features(&features_10)
                .push_next(&mut features_11)
                .push_next(&mut features_12)
                .push_next(&mut features_sync2)
                .push_next(&mut features_dynamic)
                .push_next(&mut features_divisor);
            let device = instance
                .create_device(physical_device, &device_info, None)
                .map_err(|result| VulkanError::Call("vkCreateDevice", result))?;
            let queue = device.get_device_queue(queue_family, 0);

            let surface_format = pick_surface_format(&surface_instance, physical_device, surface)?;
            let capabilities = surface_instance
                .get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|result| {
                    VulkanError::Call("vkGetPhysicalDeviceSurfaceCapabilities", result)
                })?;
            let extent = clamp_extent(&capabilities, width, height);

            let swapchain_device = ash::khr::swapchain::Device::new(&instance, &device);
            let dynamic_rendering = ash::khr::dynamic_rendering::Device::new(&instance, &device);
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(REQUESTED_IMAGE_COUNT.max(capabilities.min_image_count))
                .image_format(surface_format)
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true);
            let swapchain = swapchain_device
                .create_swapchain(&swapchain_info, None)
                .map_err(|result| VulkanError::Call("vkCreateSwapchainKHR", result))?;
            let images = swapchain_device
                .get_swapchain_images(swapchain)
                .map_err(|result| VulkanError::Call("vkGetSwapchainImagesKHR", result))?;

            let mut image_views = Vec::with_capacity(images.len());
            for image in &images {
                image_views.push(
                    device
                        .create_image_view(
                            &vk::ImageViewCreateInfo::default()
                                .image(*image)
                                .view_type(vk::ImageViewType::TYPE_2D)
                                .format(surface_format)
                                .subresource_range(
                                    vk::ImageSubresourceRange::default()
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .level_count(1)
                                        .layer_count(1),
                                ),
                            None,
                        )
                        .map_err(|result| VulkanError::Call("vkCreateImageView", result))?,
                );
            }

            let command_pool = device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .queue_family_index(queue_family)
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateCommandPool", result))?;
            let command_buffer = device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .map_err(|result| VulkanError::Call("vkAllocateCommandBuffers", result))?[0];

            let byte_count = u64::from(extent.width) * u64::from(extent.height) * 4;
            let readback_buffer = device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(byte_count)
                        .usage(vk::BufferUsageFlags::TRANSFER_DST)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = device.get_buffer_memory_requirements(readback_buffer);
            let memory_properties = instance.get_physical_device_memory_properties(physical_device);
            let memory_type = (0..memory_properties.memory_type_count)
                .find(|index| {
                    let supported = requirements.memory_type_bits & (1 << index) != 0;
                    let flags = memory_properties.memory_types[*index as usize].property_flags;
                    supported
                        && flags.contains(
                            vk::MemoryPropertyFlags::HOST_VISIBLE
                                | vk::MemoryPropertyFlags::HOST_COHERENT,
                        )
                })
                .ok_or(VulkanError::NoSuitableDevice)?;
            let readback_memory = device
                .allocate_memory(
                    &vk::MemoryAllocateInfo::default()
                        .allocation_size(requirements.size)
                        .memory_type_index(memory_type),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkAllocateMemory", result))?;
            device
                .bind_buffer_memory(readback_buffer, readback_memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;

            let image_available = device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?;
            // Present waits are per image: reusing one semaphore across images
            // is what trips validation on a FIFO swapchain.
            let mut render_finished = Vec::with_capacity(images.len());
            for _ in 0..images.len() {
                render_finished.push(
                    device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?,
                );
            }
            let in_flight = device
                .create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateFence", result))?;

            let device_name = CStr::from_ptr(properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned();
            let description = BackendDescription {
                device_name,
                discrete: properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU,
                api_version: properties.api_version,
                surface_format,
                image_count: images.len() as u32,
                extent,
            };

            Ok(Self {
                _entry: entry,
                instance,
                surface_instance,
                surface,
                physical_device,
                device,
                queue,
                swapchain_device,
                dynamic_rendering,
                swapchain,
                images,
                image_views,
                command_pool,
                command_buffer,
                readback_buffer,
                readback_memory,
                depth: None,
                out_of_date: false,
                scene: None,
                sky_buffer: None,
                pending_sky: None,
                cloud_buffer: None,
                weather_buffer: None,
                weather_counts: (0, 0),
                entity_buffer: None,
                entity_count: 0,
                sections: std::collections::BTreeMap::new(),
                arena: Vec::new(),
                staging: Vec::new(),
                staging_filling: Vec::new(),
                staging_in_flight: Vec::new(),
                staged: Vec::new(),
                retired: Vec::new(),
                sort_origin: [0.0; 3],
                lightmap: DEFAULT_LIGHTMAP,
                fog: NO_FOG,
                image_available,
                render_finished,
                in_flight,
                description,
            })
        }
    }

    /// Allocate a buffer and fill it from host memory.
    unsafe fn upload_buffer(
        &self,
        usage: vk::BufferUsageFlags,
        bytes: &[u8],
    ) -> Result<(vk::Buffer, vk::DeviceMemory), VulkanError> {
        unsafe {
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(bytes.len() as u64)
                        .usage(usage)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory = self.allocate(
                requirements,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;
            let mapped = self
                .device
                .map_memory(memory, 0, bytes.len() as u64, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped, bytes.len());
            self.device.unmap_memory(memory);
            Ok((buffer, memory))
        }
    }

    /// Find a memory type matching the requirements and properties.
    unsafe fn allocate(
        &self,
        requirements: vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory, VulkanError> {
        unsafe {
            let available = self
                .instance
                .get_physical_device_memory_properties(self.physical_device);
            let index = (0..available.memory_type_count)
                .find(|index| {
                    requirements.memory_type_bits & (1 << index) != 0
                        && available.memory_types[*index as usize]
                            .property_flags
                            .contains(properties)
                })
                .ok_or(VulkanError::NoSuitableDevice)?;
            self.device
                .allocate_memory(
                    &vk::MemoryAllocateInfo::default()
                        .allocation_size(requirements.size)
                        .memory_type_index(index),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkAllocateMemory", result))
        }
    }

    /// Run one command buffer to completion, for setup work.
    unsafe fn run_once(&self, record: impl FnOnce(vk::CommandBuffer)) -> Result<(), VulkanError> {
        unsafe {
            let buffer = self
                .device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(self.command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .map_err(|result| VulkanError::Call("vkAllocateCommandBuffers", result))?[0];
            self.device
                .begin_command_buffer(
                    buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|result| VulkanError::Call("vkBeginCommandBuffer", result))?;
            record(buffer);
            self.device
                .end_command_buffer(buffer)
                .map_err(|result| VulkanError::Call("vkEndCommandBuffer", result))?;
            let buffers = [buffer];
            let submit = vk::SubmitInfo::default().command_buffers(&buffers);
            self.device
                .queue_submit(self.queue, &[submit], vk::Fence::null())
                .map_err(|result| VulkanError::Call("vkQueueSubmit", result))?;
            self.device
                .queue_wait_idle(self.queue)
                .map_err(|result| VulkanError::Call("vkQueueWaitIdle", result))?;
            self.device
                .free_command_buffers(self.command_pool, &buffers);
            Ok(())
        }
    }

    /// Record the draw for one swapchain image.
    unsafe fn bind_section(
        &self,
        section: &SectionBuffers,
        layout: vk::PipelineLayout,
        now: std::time::Instant,
    ) {
        unsafe {
            self.device.cmd_push_constants(
                self.command_buffer,
                layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                160,
                &section.fade.visibility(now).to_ne_bytes(),
            );
            self.device.cmd_bind_vertex_buffers(
                self.command_buffer,
                0,
                &[self.arena[section.vertices.block].buffer],
                &[section.vertices.offset],
            );
            self.device.cmd_bind_index_buffer(
                self.command_buffer,
                self.arena[section.indices.block].buffer,
                section.indices.offset,
                vk::IndexType::UINT32,
            );
        }
    }

    unsafe fn record_scene(&self, index: usize, view_projection: [[f32; 4]; 4]) {
        unsafe {
            let now = std::time::Instant::now();
            let (Some(scene), Some(depth)) = (self.scene.as_ref(), self.depth.as_ref()) else {
                return;
            };
            let colour_attachment = [vk::RenderingAttachmentInfo::default()
                .image_view(self.image_views[index])
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                // The clear already happened, so the colour target is loaded.
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(depth.view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 0.0,
                        stencil: 0,
                    },
                });
            let rendering = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(self.description.extent))
                .layer_count(1)
                .color_attachments(&colour_attachment)
                .depth_attachment(&depth_attachment);
            self.dynamic_rendering
                .cmd_begin_rendering(self.command_buffer, &rendering);
            let extent = self.description.extent;
            self.device.cmd_set_viewport(
                self.command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(extent.width as f32)
                    .height(extent.height as f32)
                    .max_depth(1.0)],
            );
            self.device.cmd_set_scissor(
                self.command_buffer,
                0,
                &[vk::Rect2D::default().extent(extent)],
            );
            // The sky first, over the cleared fog colour and under everything.
            if let (Some(sky), Some(buffer)) = (self.pending_sky.as_ref(), self.sky_buffer.as_ref())
                && sky.vertex_count > 0
            {
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                let matrix: [f32; 16] = std::array::from_fn(|i| sky.view_projection[i / 4][i % 4]);
                let mut fragment = [0.0_f32; 8];
                fragment[..4].copy_from_slice(&sky.fog_colour);
                fragment[4] = sky.sky_end;
                for &(blend, first, count) in &sky.batches {
                    let (layout, pipeline) = match blend {
                        crate::render::sky::SkyBlend::Translucent => {
                            (scene.sky_pipeline_layout, scene.sky_pipeline)
                        }
                        crate::render::sky::SkyBlend::Overlay => (
                            scene.sky_overlay_pipeline_layout,
                            scene.sky_overlay_pipeline,
                        ),
                    };
                    self.device.cmd_bind_pipeline(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline,
                    );
                    self.device.cmd_bind_descriptor_sets(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        layout,
                        0,
                        &[scene.descriptor_set],
                        &[],
                    );
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        std::slice::from_raw_parts(matrix.as_ptr().cast::<u8>(), 64),
                    );
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        64,
                        std::slice::from_raw_parts(fragment.as_ptr().cast::<u8>(), 32),
                    );
                    self.device
                        .cmd_draw(self.command_buffer, count, 1, first, 0);
                }
            }
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                scene.pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                scene.pipeline_layout,
                0,
                &[scene.descriptor_set],
                &[],
            );
            let matrix: [f32; 16] = std::array::from_fn(|i| view_projection[i / 4][i % 4]);
            // The terrain shaders also take the lightmap's inputs, the
            // camera and the fog.
            let mut terrain_push = [0.0_f32; 40];
            terrain_push[..16].copy_from_slice(&matrix);
            terrain_push[16..28].copy_from_slice(&self.lightmap);
            terrain_push[28..31].copy_from_slice(&self.sort_origin);
            terrain_push[32..].copy_from_slice(&self.fog);
            self.device.cmd_push_constants(
                self.command_buffer,
                scene.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                std::slice::from_raw_parts(terrain_push.as_ptr().cast::<u8>(), 160),
            );
            for section in self
                .sections
                .values()
                .filter(|section| section.index_count > 0)
            {
                self.bind_section(section, scene.pipeline_layout, now);
                self.device
                    .cmd_draw_indexed(self.command_buffer, section.index_count, 1, 0, 0, 0);
            }
            // Entities after the opaque terrain and before the water.
            if let (Some(buffer), Some(set)) = (self.entity_buffer.as_ref(), scene.entity_set)
                && self.entity_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.entity_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.entity_pipeline_layout,
                    0,
                    &[set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.entity_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(terrain_push.as_ptr().cast::<u8>(), 160),
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.entity_count, 1, 0, 0);
            }
            // Translucent faces last, the farthest section first, so nearer
            // water blends over farther.
            let mut translucent: Vec<&SectionBuffers> = self
                .sections
                .values()
                .filter(|section| section.translucent_index_count > 0)
                .collect();
            let distance = |section: &SectionBuffers| {
                (0..3)
                    .map(|axis| (section.centre[axis] - self.sort_origin[axis]).powi(2))
                    .sum::<f32>()
            };
            translucent.sort_by(|a, b| distance(b).total_cmp(&distance(a)));
            if !translucent.is_empty() {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.translucent_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.translucent_pipeline_layout,
                    0,
                    &[scene.descriptor_set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.translucent_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(terrain_push.as_ptr().cast::<u8>(), 160),
                );
            }
            for section in translucent {
                self.bind_section(section, scene.translucent_pipeline_layout, now);
                self.device.cmd_draw_indexed(
                    self.command_buffer,
                    section.translucent_index_count,
                    1,
                    section.index_count,
                    0,
                    0,
                );
            }
            // The clouds after the terrain, tested against and writing depth.
            if let Some(clouds) = self.cloud_buffer.as_ref()
                && clouds.visible
                && clouds.vertex_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.cloud_pipeline,
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.cloud_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(clouds.push.as_ptr().cast::<u8>(), 96),
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[clouds.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, clouds.vertex_count, 1, 0, 0);
            }
            // Rain, then snow, each with its texture, over everything and
            // leaving the depth alone.
            if let Some(buffer) = self.weather_buffer.as_ref()
                && scene.weather_sets.len() == 2
                && self.weather_counts.0 + self.weather_counts.1 > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.weather_pipeline,
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.weather_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(terrain_push.as_ptr().cast::<u8>(), 160),
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                let (rain, snow) = self.weather_counts;
                for (set, first, count) in [
                    (scene.weather_sets[0], 0, rain),
                    (scene.weather_sets[1], rain, snow),
                ] {
                    if count == 0 {
                        continue;
                    }
                    self.device.cmd_bind_descriptor_sets(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.weather_pipeline_layout,
                        0,
                        &[set],
                        &[],
                    );
                    self.device
                        .cmd_draw(self.command_buffer, count, 1, first, 0);
                }
            }
            self.dynamic_rendering
                .cmd_end_rendering(self.command_buffer);
        }
    }

    /// Compile the shaders and build the graphics pipeline.
    ///
    /// Dynamic rendering is used rather than a render pass object, which is
    /// the extension the reference already requires of every device it picks.
    unsafe fn create_pipeline(
        &self,
        descriptor_layout: vk::DescriptorSetLayout,
        kind: PipelineKind,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline), VulkanError> {
        let sky = matches!(
            kind,
            PipelineKind::SkyTranslucent | PipelineKind::SkyOverlay
        );
        let translucent = kind == PipelineKind::Translucent;
        let clouds = kind == PipelineKind::Clouds;
        let weather = kind == PipelineKind::Weather;
        let entity = kind == PipelineKind::Entity;
        unsafe {
            let compiler =
                shaderc::Compiler::new().map_err(|error| VulkanError::Loader(error.to_string()))?;
            let compile = |source: &str, kind, name: &str| -> Result<Vec<u32>, VulkanError> {
                compiler
                    .compile_into_spirv(source, kind, name, "main", None)
                    .map(|artifact| artifact.as_binary().to_vec())
                    .map_err(|error| VulkanError::Loader(error.to_string()))
            };
            let (vertex_source, fragment_source) = if sky {
                (SKY_VERTEX_SHADER, SKY_FRAGMENT_SHADER)
            } else if clouds {
                (CLOUD_VERTEX_SHADER, CLOUD_FRAGMENT_SHADER)
            } else if weather {
                (VERTEX_SHADER, WEATHER_FRAGMENT_SHADER)
            } else if entity {
                (ENTITY_VERTEX_SHADER, ENTITY_FRAGMENT_SHADER)
            } else {
                (VERTEX_SHADER, FRAGMENT_SHADER)
            };
            let vertex_code = compile(vertex_source, shaderc::ShaderKind::Vertex, "scene.vert")?;
            let fragment_code =
                compile(fragment_source, shaderc::ShaderKind::Fragment, "scene.frag")?;
            let make_module = |code: &[u32]| -> Result<vk::ShaderModule, VulkanError> {
                self.device
                    .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None)
                    .map_err(|result| VulkanError::Call("vkCreateShaderModule", result))
            };
            let vertex_module = make_module(&vertex_code)?;
            let fragment_module = make_module(&fragment_code)?;

            let entry = CString::new("main").expect("static name");
            let stages = [
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::VERTEX)
                    .module(vertex_module)
                    .name(&entry),
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(fragment_module)
                    .name(&entry),
            ];

            let stride = if sky {
                std::mem::size_of::<crate::render::sky::SkyVertex>()
            } else if clouds {
                std::mem::size_of::<crate::render::clouds::CloudVertex>()
            } else {
                std::mem::size_of::<Vertex>()
            };
            let bindings = [vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(stride as u32)
                .input_rate(vk::VertexInputRate::VERTEX)];
            let sky_attributes = [
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(12),
                vk::VertexInputAttributeDescription::default()
                    .location(2)
                    .binding(0)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(20),
                vk::VertexInputAttributeDescription::default()
                    .location(3)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(36),
            ];
            let terrain_attributes = [
                vk::VertexInputAttributeDescription::default()
                    .location(4)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(40),
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(12),
                vk::VertexInputAttributeDescription::default()
                    .location(2)
                    .binding(0)
                    .format(vk::Format::R32_SFLOAT)
                    .offset(20),
                vk::VertexInputAttributeDescription::default()
                    .location(3)
                    .binding(0)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(24),
            ];
            let cloud_attributes = [
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32_SFLOAT)
                    .offset(12),
            ];
            let attributes: &[vk::VertexInputAttributeDescription] = if sky {
                &sky_attributes
            } else if clouds {
                &cloud_attributes
            } else {
                &terrain_attributes
            };
            let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&bindings)
                .vertex_attribute_descriptions(attributes);
            let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
            // The viewport follows the window, so it is set per frame.
            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewport_count(1)
                .scissor_count(1);
            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state =
                vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
            let raster = vk::PipelineRasterizationStateCreateInfo::default()
                .polygon_mode(vk::PolygonMode::FILL)
                // The sky's fans are built in either winding.
                .cull_mode(if sky || weather || entity {
                    vk::CullModeFlags::NONE
                } else {
                    vk::CullModeFlags::BACK
                })
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .line_width(1.0);
            let multisample = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let depth_state = vk::PipelineDepthStencilStateCreateInfo::default()
                // The sky is behind everything and tests nothing. Translucent
                // faces are tested against the opaque depth but leave it
                // alone, so what is behind them still shows. Depth is
                // reversed and equal depth passes, as in the reference, so
                // a model's later face over an earlier one at the same place
                // (the grass side overlay) is the one seen.
                .depth_test_enable(!sky)
                .depth_write_enable(matches!(
                    kind,
                    PipelineKind::Terrain | PipelineKind::Clouds | PipelineKind::Entity
                ))
                .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
            let blends = [if kind == PipelineKind::SkyOverlay {
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if translucent || sky || clouds || weather || entity {
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else {
                vk::PipelineColorBlendAttachmentState::default()
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            }];
            let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blends);

            let push_ranges = [
                if sky {
                    vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX)
                        .size(64)
                } else if clouds {
                    vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                        .size(96)
                } else {
                    vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                        .size(164)
                },
                vk::PushConstantRange::default()
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .offset(64)
                    .size(32),
            ];
            let push_ranges = if sky {
                &push_ranges[..]
            } else {
                &push_ranges[..1]
            };
            let set_layouts = [descriptor_layout];
            let pipeline_layout = self
                .device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::default()
                        .set_layouts(&set_layouts)
                        .push_constant_ranges(push_ranges),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreatePipelineLayout", result))?;

            let colour_formats = [self.description.surface_format];
            let mut rendering = vk::PipelineRenderingCreateInfo::default()
                .color_attachment_formats(&colour_formats)
                .depth_attachment_format(
                    self.depth
                        .as_ref()
                        .map_or(vk::Format::UNDEFINED, |d| d.format),
                );
            let create = vk::GraphicsPipelineCreateInfo::default()
                .stages(&stages)
                .vertex_input_state(&vertex_input)
                .input_assembly_state(&assembly)
                .viewport_state(&viewport_state)
                .dynamic_state(&dynamic_state)
                .rasterization_state(&raster)
                .multisample_state(&multisample)
                .depth_stencil_state(&depth_state)
                .color_blend_state(&blend)
                .layout(pipeline_layout)
                .push_next(&mut rendering);
            let pipeline = self
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create], None)
                .map_err(|(_, result)| VulkanError::Call("vkCreateGraphicsPipelines", result))?[0];

            self.device.destroy_shader_module(vertex_module, None);
            self.device.destroy_shader_module(fragment_module, None);
            Ok((pipeline_layout, pipeline))
        }
    }

    /// Build the depth target, texture, buffers and pipeline for one mesh.
    ///
    /// Shaders are compiled here with shaderc, which is the library the
    /// reference's Vulkan backend uses; the GLSL itself is this project's own.
    pub fn load_scene(
        &mut self,
        mesh: &Mesh,
        texture_levels: &[DecodedImage],
    ) -> Result<(), VulkanError> {
        self.load_atlas(texture_levels)?;
        self.set_section((0, 0, 0), mesh, [0.0; 3], SectionFade::none())
    }

    /// Replace one section's mesh, or remove it when the mesh is empty.
    pub fn set_section(
        &mut self,
        key: (i32, i32, i32),
        mesh: &Mesh,
        centre: [f32; 3],
        fade: SectionFade,
    ) -> Result<(), VulkanError> {
        unsafe {
            // The frame in flight may still read the old mesh, so it is
            // freed after that frame.
            if let Some(old) = self.sections.remove(&key) {
                self.retired.push(old);
            }
            if mesh.indices.is_empty() && mesh.translucent_indices.is_empty() {
                return Ok(());
            }
            let vertex_bytes = std::slice::from_raw_parts(
                mesh.vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(mesh.vertices.as_slice()),
            );
            let vertices = self.arena_upload(vertex_bytes)?;
            let all_indices: Vec<u32> = mesh
                .indices
                .iter()
                .chain(&mesh.translucent_indices)
                .copied()
                .collect();
            let index_bytes = std::slice::from_raw_parts(
                all_indices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(all_indices.as_slice()),
            );
            let indices = self.arena_upload(index_bytes)?;
            self.sections.insert(
                key,
                SectionBuffers {
                    vertices,
                    indices,
                    index_count: mesh.indices.len() as u32,
                    translucent_index_count: mesh.translucent_indices.len() as u32,
                    centre,
                    fade,
                },
            );
            Ok(())
        }
    }

    /// Whether the swapchain no longer fits the window and wants
    /// [`Self::resize`].
    pub fn out_of_date(&self) -> bool {
        self.out_of_date
    }

    /// The swapchain's size in pixels.
    pub fn extent(&self) -> (u32, u32) {
        (
            self.description.extent.width,
            self.description.extent.height,
        )
    }

    /// Rebuild the swapchain, and what is sized by it, for a new window
    /// size. A zero size (a minimised window) is left for later.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), VulkanError> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|result| VulkanError::Call("vkDeviceWaitIdle", result))?;
            let capabilities = self
                .surface_instance
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
                .map_err(|result| {
                    VulkanError::Call("vkGetPhysicalDeviceSurfaceCapabilities", result)
                })?;
            let extent = clamp_extent(&capabilities, width, height);
            if extent.width == 0 || extent.height == 0 {
                return Ok(());
            }
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .surface(self.surface)
                .min_image_count(REQUESTED_IMAGE_COUNT.max(capabilities.min_image_count))
                .image_format(self.description.surface_format)
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true)
                .old_swapchain(self.swapchain);
            let swapchain = self
                .swapchain_device
                .create_swapchain(&swapchain_info, None)
                .map_err(|result| VulkanError::Call("vkCreateSwapchainKHR", result))?;
            for view in self.image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
            self.swapchain = swapchain;
            self.images = self
                .swapchain_device
                .get_swapchain_images(swapchain)
                .map_err(|result| VulkanError::Call("vkGetSwapchainImagesKHR", result))?;
            for image in &self.images {
                self.image_views.push(
                    self.device
                        .create_image_view(
                            &vk::ImageViewCreateInfo::default()
                                .image(*image)
                                .view_type(vk::ImageViewType::TYPE_2D)
                                .format(self.description.surface_format)
                                .subresource_range(
                                    vk::ImageSubresourceRange::default()
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .level_count(1)
                                        .layer_count(1),
                                ),
                            None,
                        )
                        .map_err(|result| VulkanError::Call("vkCreateImageView", result))?,
                );
            }
            for semaphore in self.render_finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            for _ in 0..self.images.len() {
                self.render_finished.push(
                    self.device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?,
                );
            }
            // The acquire semaphore may have been left signalled by a failed
            // acquire, so it is replaced too.
            self.device.destroy_semaphore(self.image_available, None);
            self.image_available = self
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?;
            self.description.extent = extent;
            self.description.image_count = self.images.len() as u32;

            self.device.destroy_buffer(self.readback_buffer, None);
            self.device.free_memory(self.readback_memory, None);
            self.readback_buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(u64::from(extent.width) * u64::from(extent.height) * 4)
                        .usage(vk::BufferUsageFlags::TRANSFER_DST)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = self
                .device
                .get_buffer_memory_requirements(self.readback_buffer);
            self.readback_memory = self.allocate(
                requirements,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            self.device
                .bind_buffer_memory(self.readback_buffer, self.readback_memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;

            if let Some(depth) = self.depth.take() {
                self.device.destroy_image_view(depth.view, None);
                self.device.destroy_image(depth.image, None);
                self.device.free_memory(depth.memory, None);
                self.depth = Some(self.create_depth_target()?);
            }
            self.out_of_date = false;
            Ok(())
        }
    }

    /// Drop every section mesh.
    pub fn clear_sections(&mut self) -> Result<(), VulkanError> {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            for (_, section) in std::mem::take(&mut self.sections) {
                self.destroy_section(section);
            }
            for section in std::mem::take(&mut self.retired) {
                self.destroy_section(section);
            }
            // Their copies have nowhere to go now.
            self.staged.clear();
            self.staging_filling.clear();
            Ok(())
        }
    }

    /// Where translucent sections are ordered from, usually the camera.
    pub fn set_sort_origin(&mut self, origin: [f32; 3]) {
        self.sort_origin = origin;
    }

    /// Give a section's ranges back to the arena; the GPU must be done
    /// with them.
    unsafe fn destroy_section(&mut self, section: SectionBuffers) {
        self.arena_free(section.vertices);
        self.arena_free(section.indices);
    }

    /// Copy bytes into a free range of the arena, growing it by a block
    /// when none fits.
    unsafe fn arena_upload(&mut self, bytes: &[u8]) -> Result<ArenaSlice, VulkanError> {
        unsafe {
            let size = (bytes.len() as u64).div_ceil(16) * 16;
            let found = self.arena.iter().enumerate().find_map(|(block, arena)| {
                arena
                    .free
                    .iter()
                    .position(|&(_, length)| length >= size)
                    .map(|range| (block, range))
            });
            let (block, range) = match found {
                Some(found) => found,
                None => {
                    self.arena
                        .push(self.create_arena_block(size.max(ARENA_BLOCK_SIZE))?);
                    (self.arena.len() - 1, 0)
                }
            };
            let arena = &mut self.arena[block];
            let (offset, length) = arena.free[range];
            if length == size {
                arena.free.remove(range);
            } else {
                arena.free[range] = (offset + size, length - size);
            }
            let target = ArenaSlice {
                block,
                offset,
                size,
            };
            let chunk = match self
                .staging_filling
                .iter()
                .copied()
                .find(|&chunk| self.staging[chunk].size - self.staging[chunk].used >= size)
            {
                Some(chunk) => chunk,
                None => {
                    let free = (0..self.staging.len()).find(|chunk| {
                        !self.staging_filling.contains(chunk)
                            && !self.staging_in_flight.contains(chunk)
                            && self.staging[*chunk].size >= size
                    });
                    let chunk = match free {
                        Some(chunk) => chunk,
                        None => {
                            self.staging
                                .push(self.create_staging_chunk(size.max(STAGING_CHUNK_SIZE))?);
                            self.staging.len() - 1
                        }
                    };
                    self.staging[chunk].used = 0;
                    self.staging_filling.push(chunk);
                    chunk
                }
            };
            let staging = &mut self.staging[chunk];
            let source = staging.used;
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                staging.mapped.add(source as usize),
                bytes.len(),
            );
            staging.used += size;
            self.staged.push(StagedCopy {
                chunk,
                source,
                target,
                size,
            });
            Ok(target)
        }
    }

    fn arena_free(&mut self, slice: ArenaSlice) {
        let free = &mut self.arena[slice.block].free;
        let at = free.partition_point(|&(offset, _)| offset < slice.offset);
        free.insert(at, (slice.offset, slice.size));
        // Join with the ranges on either side.
        if at + 1 < free.len() && free[at].0 + free[at].1 == free[at + 1].0 {
            free[at].1 += free[at + 1].1;
            free.remove(at + 1);
        }
        if at > 0 && free[at - 1].0 + free[at - 1].1 == free[at].0 {
            free[at - 1].1 += free[at].1;
            free.remove(at);
        }
    }

    /// A new arena block, in the GPU's own memory.
    unsafe fn create_arena_block(&self, size: u64) -> Result<ArenaBlock, VulkanError> {
        unsafe {
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(size)
                        .usage(
                            vk::BufferUsageFlags::VERTEX_BUFFER
                                | vk::BufferUsageFlags::INDEX_BUFFER
                                | vk::BufferUsageFlags::TRANSFER_DST,
                        )
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory = self.allocate(requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;
            Ok(ArenaBlock {
                buffer,
                memory,
                free: vec![(0, size)],
            })
        }
    }

    /// A new staging chunk, mapped for good.
    unsafe fn create_staging_chunk(&self, size: u64) -> Result<StagingChunk, VulkanError> {
        unsafe {
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(size)
                        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory = self.allocate(
                requirements,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;
            let mapped = self
                .device
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            Ok(StagingChunk {
                buffer,
                memory,
                mapped,
                size,
                used: 0,
            })
        }
    }

    /// Record the staged section copies, and make the draws wait for them.
    unsafe fn record_staged_copies(&mut self) {
        unsafe {
            if self.staged.is_empty() {
                return;
            }
            for copy in std::mem::take(&mut self.staged) {
                self.device.cmd_copy_buffer(
                    self.command_buffer,
                    self.staging[copy.chunk].buffer,
                    self.arena[copy.target.block].buffer,
                    &[vk::BufferCopy {
                        src_offset: copy.source,
                        dst_offset: copy.target.offset,
                        size: copy.size,
                    }],
                );
            }
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::VERTEX_INPUT,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(
                        vk::AccessFlags::VERTEX_ATTRIBUTE_READ | vk::AccessFlags::INDEX_READ,
                    )],
                &[],
                &[],
            );
            self.staging_in_flight
                .extend(std::mem::take(&mut self.staging_filling));
        }
    }

    /// Upload the atlas and build the pipelines, dropping every section.
    pub fn load_atlas(&mut self, texture_levels: &[DecodedImage]) -> Result<(), VulkanError> {
        unsafe {
            self.destroy_scene();
            self.depth = Some(self.create_depth_target()?);

            let (texture, texture_memory, texture_view) = self.upload_texture(texture_levels)?;
            // These are the reference's own values for the terrain sampler,
            // read from `LevelRenderer`'s chunk layer sampler and from the
            // Vulkan sampler it builds. Both filters are linear, which reads
            // as a surprise for a game of pixel art but is consistent with the
            // atlas reserving padding around every sprite: with nearest
            // sampling that padding would have nothing to do. The blocky look
            // comes from the fragment shader instead, which moves each lookup
            // to its texel's centre before the linear filter sees it.
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::LINEAR)
                        .min_filter(vk::Filter::LINEAR)
                        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        // The reference leaves the third axis at the default,
                        // which is repeat, and never samples it.
                        .address_mode_w(vk::SamplerAddressMode::REPEAT)
                        .mip_lod_bias(0.0)
                        .min_lod(0.0)
                        // An unbounded maximum, which the reference spells as
                        // one thousand.
                        .max_lod(1000.0)
                        .anisotropy_enable(false)
                        .max_anisotropy(1.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;

            let bindings = [vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
            let descriptor_layout = self
                .device
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateDescriptorSetLayout", result))?;
            // The atlas's set, the rain's and the snow's, and the skin's.
            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(4)];
            let descriptor_pool = self
                .device
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo::default()
                        .max_sets(4)
                        .pool_sizes(&sizes),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateDescriptorPool", result))?;
            let layouts = [descriptor_layout];
            let descriptor_set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(descriptor_pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_view)
                .sampler(sampler)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);

            let (pipeline_layout, pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Terrain)?;
            let (translucent_pipeline_layout, translucent_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Translucent)?;
            let (sky_pipeline_layout, sky_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::SkyTranslucent)?;
            let (sky_overlay_pipeline_layout, sky_overlay_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::SkyOverlay)?;
            let (cloud_pipeline_layout, cloud_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Clouds)?;
            let (weather_pipeline_layout, weather_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Weather)?;
            let (entity_pipeline_layout, entity_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Entity)?;

            self.scene = Some(SceneResources {
                texture,
                texture_memory,
                texture_view,
                sampler,
                descriptor_pool,
                descriptor_layout,
                descriptor_set,
                pipeline_layout,
                pipeline,
                translucent_pipeline_layout,
                translucent_pipeline,
                sky_pipeline_layout,
                sky_pipeline,
                sky_overlay_pipeline_layout,
                sky_overlay_pipeline,
                cloud_pipeline_layout,
                cloud_pipeline,
                weather_pipeline_layout,
                weather_pipeline,
                weather_textures: Vec::new(),
                weather_sampler: None,
                weather_sets: Vec::new(),
                entity_pipeline_layout,
                entity_pipeline,
                entity_texture: None,
                entity_sampler: None,
                entity_set: None,
            });
            Ok(())
        }
    }

    unsafe fn create_depth_target(&self) -> Result<DepthTarget, VulkanError> {
        unsafe {
            let format = vk::Format::D32_SFLOAT;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: self.description.extent.width,
                            height: self.description.extent.height,
                            depth: 1,
                        })
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .initial_layout(vk::ImageLayout::UNDEFINED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImage", result))?;
            let requirements = self.device.get_image_memory_requirements(image);
            let memory = self.allocate(requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.device
                .bind_image_memory(image, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindImageMemory", result))?;
            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .level_count(1)
                                .layer_count(1),
                        ),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            Ok(DepthTarget {
                image,
                memory,
                view,
                format,
            })
        }
    }

    /// Rewrite rectangles of the loaded atlas: each a level, a corner, a
    /// size and its tightly packed RGBA8 texels. Animated sprites use it
    /// every tick their frame changes.
    pub fn update_atlas(&mut self, regions: &[AtlasRegion]) -> Result<(), VulkanError> {
        let Some(scene) = self.scene.as_ref() else {
            return Ok(());
        };
        if regions.is_empty() {
            return Ok(());
        }
        unsafe {
            // The previous frame may still be sampling the atlas.
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let mut staged = Vec::new();
            let mut copies = Vec::with_capacity(regions.len());
            let mut deepest = 0;
            for region in regions {
                copies.push(
                    vk::BufferImageCopy::default()
                        .buffer_offset(staged.len() as u64)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(region.level)
                                .layer_count(1),
                        )
                        .image_offset(vk::Offset3D {
                            x: region.x as i32,
                            y: region.y as i32,
                            z: 0,
                        })
                        .image_extent(vk::Extent3D {
                            width: region.width,
                            height: region.height,
                            depth: 1,
                        }),
                );
                staged.extend_from_slice(&region.rgba);
                deepest = deepest.max(region.level);
            }
            let (staging, staging_memory) =
                self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
            let image = scene.texture;
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(deepest + 1)
                .layer_count(1);
            let result = self.run_once(|command| {
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
                self.device.cmd_copy_buffer_to_image(
                    command,
                    staging,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &copies,
                );
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
            });
            self.device.destroy_buffer(staging, None);
            self.device.free_memory(staging_memory, None);
            result
        }
    }

    unsafe fn upload_texture(
        &self,
        levels: &[DecodedImage],
    ) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), VulkanError> {
        unsafe {
            let base = levels.first().ok_or(VulkanError::NoAtlasLevels)?;
            let mut staged = Vec::new();
            let mut offsets = Vec::with_capacity(levels.len());
            for level in levels {
                offsets.push(staged.len() as u64);
                staged.extend_from_slice(&level.rgba);
            }
            let (staging, staging_memory) =
                self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
            let format = vk::Format::R8G8B8A8_UNORM;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: base.width,
                            height: base.height,
                            depth: 1,
                        })
                        .mip_levels(levels.len() as u32)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .initial_layout(vk::ImageLayout::UNDEFINED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImage", result))?;
            let requirements = self.device.get_image_memory_requirements(image);
            let memory = self.allocate(requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.device
                .bind_image_memory(image, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindImageMemory", result))?;

            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(levels.len() as u32)
                .layer_count(1);
            // Every level is already built on the processor, so nothing is
            // blitted here: each one is its own copy out of the staging buffer.
            let regions: Vec<vk::BufferImageCopy> = levels
                .iter()
                .zip(&offsets)
                .enumerate()
                .map(|(level, (image, offset))| {
                    vk::BufferImageCopy::default()
                        .buffer_offset(*offset)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(level as u32)
                                .layer_count(1),
                        )
                        .image_extent(vk::Extent3D {
                            width: image.width,
                            height: image.height,
                            depth: 1,
                        })
                })
                .collect();
            self.run_once(|command| {
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
                self.device.cmd_copy_buffer_to_image(
                    command,
                    staging,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &regions,
                );
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
            })?;
            self.device.destroy_buffer(staging, None);
            self.device.free_memory(staging_memory, None);

            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(range),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            Ok((image, memory, view))
        }
    }

    unsafe fn destroy_scene(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            if let Some(scene) = self.scene.take() {
                self.device.destroy_pipeline(scene.pipeline, None);
                self.device
                    .destroy_pipeline(scene.translucent_pipeline, None);
                self.device.destroy_pipeline(scene.sky_pipeline, None);
                self.device
                    .destroy_pipeline_layout(scene.sky_pipeline_layout, None);
                self.device
                    .destroy_pipeline(scene.sky_overlay_pipeline, None);
                self.device
                    .destroy_pipeline_layout(scene.sky_overlay_pipeline_layout, None);
                self.device.destroy_pipeline(scene.cloud_pipeline, None);
                self.device
                    .destroy_pipeline_layout(scene.cloud_pipeline_layout, None);
                self.device.destroy_pipeline(scene.weather_pipeline, None);
                self.device
                    .destroy_pipeline_layout(scene.weather_pipeline_layout, None);
                for (image, memory, view) in scene.weather_textures {
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                if let Some(sampler) = scene.weather_sampler {
                    self.device.destroy_sampler(sampler, None);
                }
                self.device.destroy_pipeline(scene.entity_pipeline, None);
                self.device
                    .destroy_pipeline_layout(scene.entity_pipeline_layout, None);
                if let Some((image, memory, view)) = scene.entity_texture {
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                if let Some(sampler) = scene.entity_sampler {
                    self.device.destroy_sampler(sampler, None);
                }
                self.device
                    .destroy_pipeline_layout(scene.translucent_pipeline_layout, None);
                self.device
                    .destroy_pipeline_layout(scene.pipeline_layout, None);
                self.device
                    .destroy_descriptor_pool(scene.descriptor_pool, None);
                self.device
                    .destroy_descriptor_set_layout(scene.descriptor_layout, None);
                self.device.destroy_sampler(scene.sampler, None);
                self.device.destroy_image_view(scene.texture_view, None);
                self.device.destroy_image(scene.texture, None);
                self.device.free_memory(scene.texture_memory, None);
            }
            for (_, section) in std::mem::take(&mut self.sections) {
                self.destroy_section(section);
            }
            for section in std::mem::take(&mut self.retired) {
                self.destroy_section(section);
            }
            // Their copies have nowhere to go now.
            self.staged.clear();
            self.staging_filling.clear();
            if let Some(depth) = self.depth.take() {
                self.device.destroy_image_view(depth.view, None);
                self.device.destroy_image(depth.image, None);
                self.device.free_memory(depth.memory, None);
            }
        }
    }

    /// What the backend selected.
    pub fn description(&self) -> &BackendDescription {
        &self.description
    }

    /// Acquire, clear to one colour, and present a single frame.
    ///
    /// The colour is a placeholder: no reference pixel has been compared yet.
    pub fn draw_clear_frame(&mut self, colour: [f32; 4]) -> Result<(), VulkanError> {
        self.render_frame(colour, false, IDENTITY).map(|_| ())
    }

    /// Draw the loaded scene under one view-projection matrix.
    pub fn draw_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<(), VulkanError> {
        self.render_frame(colour, false, view_projection)
            .map(|_| ())
    }

    /// Set the lightmap's inputs for the frames that follow: the sky light
    /// colour and factor, the block light tint and factor, and the ambient
    /// colour and brightness option.
    pub fn set_lightmap(
        &mut self,
        sky_colour: [f32; 3],
        sky_factor: f32,
        block_tint: [f32; 3],
        block_factor: f32,
        ambient: [f32; 3],
        brightness: f32,
    ) {
        self.lightmap = [
            sky_colour[0],
            sky_colour[1],
            sky_colour[2],
            sky_factor,
            block_tint[0],
            block_tint[1],
            block_tint[2],
            block_factor,
            ambient[0],
            ambient[1],
            ambient[2],
            brightness,
        ];
    }

    /// Set the terrain fog for the frames that follow: its colour, and the
    /// environmental start and end and the render distance start and end,
    /// in blocks.
    pub fn set_fog(&mut self, colour: [f32; 4], distances: [f32; 4]) {
        self.fog[..4].copy_from_slice(&colour);
        self.fog[4..].copy_from_slice(&distances);
    }

    /// Load the rain and snow textures, sampled at the nearest texel and
    /// repeating, as the reference's plain resource textures are.
    pub fn load_weather_textures(
        &mut self,
        rain: &DecodedImage,
        snow: &DecodedImage,
    ) -> Result<(), VulkanError> {
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptor_layout) else {
                return Ok(());
            };
            let pool = self.scene.as_ref().expect("checked above").descriptor_pool;
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::NEAREST)
                        .min_filter(vk::Filter::NEAREST)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::REPEAT)
                        .address_mode_v(vk::SamplerAddressMode::REPEAT)
                        .address_mode_w(vk::SamplerAddressMode::REPEAT)
                        .max_lod(0.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let mut textures = Vec::new();
            let mut sets = Vec::new();
            for image in [rain, snow] {
                let (texture, memory, view) = self.upload_texture(std::slice::from_ref(image))?;
                let layouts = [layout];
                let set = self
                    .device
                    .allocate_descriptor_sets(
                        &vk::DescriptorSetAllocateInfo::default()
                            .descriptor_pool(pool)
                            .set_layouts(&layouts),
                    )
                    .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
                let image_info = [vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(view)
                    .sampler(sampler)];
                let write = vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info);
                self.device.update_descriptor_sets(&[write], &[]);
                textures.push((texture, memory, view));
                sets.push(set);
            }
            let scene = self.scene.as_mut().expect("checked above");
            scene.weather_textures = textures;
            scene.weather_sampler = Some(sampler);
            scene.weather_sets = sets;
            Ok(())
        }
    }

    /// Load the skin entities are drawn with, sampled texel by texel.
    pub fn load_entity_texture(&mut self, skin: &DecodedImage) -> Result<(), VulkanError> {
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptor_layout) else {
                return Ok(());
            };
            if self
                .scene
                .as_ref()
                .is_some_and(|scene| scene.entity_set.is_some())
            {
                return Ok(());
            }
            let pool = self.scene.as_ref().expect("checked above").descriptor_pool;
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::NEAREST)
                        .min_filter(vk::Filter::NEAREST)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .max_lod(0.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let (texture, memory, view) = self.upload_texture(std::slice::from_ref(skin))?;
            let layouts = [layout];
            let set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(view)
                .sampler(sampler)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);
            let scene = self.scene.as_mut().expect("checked above");
            scene.entity_texture = Some((texture, memory, view));
            scene.entity_sampler = Some(sampler);
            scene.entity_set = Some(set);
            Ok(())
        }
    }

    /// Hand over the next frame's entity triangles, placed like the
    /// terrain's, with each vertex's normal in its tint.
    pub fn set_entities(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let size = std::mem::size_of_val(vertices);
            if self
                .entity_buffer
                .as_ref()
                .is_none_or(|buffer| buffer.capacity < size)
            {
                if let Some(old) = self.entity_buffer.take() {
                    self.device.destroy_buffer(old.buffer, None);
                    self.device.free_memory(old.memory, None);
                }
                let capacity = size.max(1 << 16);
                let (buffer, memory) =
                    self.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vec![0_u8; capacity])?;
                self.entity_buffer = Some(SkyBuffer {
                    buffer,
                    memory,
                    capacity,
                });
            }
            let buffer = self.entity_buffer.as_ref().expect("allocated above");
            if size > 0 {
                let mapped = self
                    .device
                    .map_memory(buffer.memory, 0, size as u64, vk::MemoryMapFlags::empty())
                    .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(vertices.as_ptr().cast::<u8>(), mapped, size);
                self.device.unmap_memory(buffer.memory);
            }
            self.entity_count = vertices.len() as u32;
            Ok(())
        }
    }

    /// Hand over the next frame's rain and snow triangles, placed like the
    /// terrain's.
    pub fn set_weather(&mut self, rain: &[Vertex], snow: &[Vertex]) -> Result<(), VulkanError> {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let size = std::mem::size_of::<Vertex>() * (rain.len() + snow.len());
            if self
                .weather_buffer
                .as_ref()
                .is_none_or(|buffer| buffer.capacity < size)
            {
                if let Some(old) = self.weather_buffer.take() {
                    self.device.destroy_buffer(old.buffer, None);
                    self.device.free_memory(old.memory, None);
                }
                let capacity = size.max(1 << 16);
                let (buffer, memory) =
                    self.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vec![0_u8; capacity])?;
                self.weather_buffer = Some(SkyBuffer {
                    buffer,
                    memory,
                    capacity,
                });
            }
            let buffer = self.weather_buffer.as_ref().expect("allocated above");
            if size > 0 {
                let mapped = self
                    .device
                    .map_memory(buffer.memory, 0, size as u64, vk::MemoryMapFlags::empty())
                    .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                    .cast::<u8>();
                let rain_bytes = std::mem::size_of_val(rain);
                std::ptr::copy_nonoverlapping(rain.as_ptr().cast::<u8>(), mapped, rain_bytes);
                std::ptr::copy_nonoverlapping(
                    snow.as_ptr().cast::<u8>(),
                    mapped.add(rain_bytes),
                    std::mem::size_of_val(snow),
                );
                self.device.unmap_memory(buffer.memory);
            }
            self.weather_counts = (rain.len() as u32, snow.len() as u32);
            Ok(())
        }
    }

    /// Replace the clouds' faces, placed from the camera's cell corner.
    pub fn set_cloud_faces(
        &mut self,
        vertices: &[crate::render::clouds::CloudVertex],
    ) -> Result<(), VulkanError> {
        unsafe {
            // The previous frame may still be reading the buffer.
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let bytes = std::slice::from_raw_parts(
                vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(vertices),
            );
            if self
                .cloud_buffer
                .as_ref()
                .is_none_or(|buffer| buffer.capacity < bytes.len())
            {
                if let Some(old) = self.cloud_buffer.take() {
                    self.device.destroy_buffer(old.buffer, None);
                    self.device.free_memory(old.memory, None);
                }
                let capacity = bytes.len().max(1 << 20);
                let (buffer, memory) =
                    self.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vec![0_u8; capacity])?;
                self.cloud_buffer = Some(CloudBuffer {
                    buffer,
                    memory,
                    capacity,
                    vertex_count: 0,
                    push: [0.0; 24],
                    visible: false,
                });
            }
            let buffer = self.cloud_buffer.as_mut().expect("allocated above");
            if !bytes.is_empty() {
                let mapped = self
                    .device
                    .map_memory(
                        buffer.memory,
                        0,
                        bytes.len() as u64,
                        vk::MemoryMapFlags::empty(),
                    )
                    .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped, bytes.len());
                self.device.unmap_memory(buffer.memory);
            }
            buffer.vertex_count = vertices.len() as u32;
            Ok(())
        }
    }

    /// Place the clouds for the frames that follow: the camera-centred
    /// view-projection, the camera's cell corner from the camera, where the
    /// cloud fog ends and the cloud colour; or hide them.
    pub fn set_cloud_frame(&mut self, placement: Option<CloudFrame>) {
        let Some(clouds) = self.cloud_buffer.as_mut() else {
            return;
        };
        clouds.visible = placement.is_some();
        if let Some(CloudFrame {
            view_projection,
            offset,
            fog_end: end,
            colour,
        }) = placement
        {
            for (index, value) in clouds.push[..16].iter_mut().enumerate() {
                *value = view_projection[index / 4][index % 4];
            }
            clouds.push[16..19].copy_from_slice(&offset);
            clouds.push[19] = end;
            clouds.push[20..].copy_from_slice(&colour);
        }
    }

    /// Hand over the next frame's sky: triangles in view-rotated space, the
    /// runs they draw in, the rotation-only view-projection, and the fog.
    pub fn set_sky(
        &mut self,
        vertices: &[crate::render::sky::SkyVertex],
        batches: &[(crate::render::sky::SkyBlend, u32, u32)],
        view_projection: [[f32; 4]; 4],
        fog_colour: [f32; 4],
        sky_end: f32,
    ) -> Result<(), VulkanError> {
        unsafe {
            // The previous frame may still be reading the buffer.
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let bytes = std::slice::from_raw_parts(
                vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(vertices),
            );
            if self
                .sky_buffer
                .as_ref()
                .is_none_or(|buffer| buffer.capacity < bytes.len())
            {
                if let Some(old) = self.sky_buffer.take() {
                    self.device.destroy_buffer(old.buffer, None);
                    self.device.free_memory(old.memory, None);
                }
                let capacity = bytes.len().max(1 << 20);
                let (buffer, memory) =
                    self.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vec![0_u8; capacity])?;
                self.sky_buffer = Some(SkyBuffer {
                    buffer,
                    memory,
                    capacity,
                });
            }
            let buffer = self.sky_buffer.as_ref().expect("allocated above");
            if !bytes.is_empty() {
                let mapped = self
                    .device
                    .map_memory(
                        buffer.memory,
                        0,
                        bytes.len() as u64,
                        vk::MemoryMapFlags::empty(),
                    )
                    .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped, bytes.len());
                self.device.unmap_memory(buffer.memory);
            }
            self.pending_sky = Some(PendingSky {
                vertex_count: vertices.len() as u32,
                batches: batches.to_vec(),
                view_projection,
                fog_colour,
                sky_end,
            });
            Ok(())
        }
    }

    /// Draw the loaded scene and read the presented image back.
    pub fn capture_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, view_projection)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    /// Render one frame and read the presented image back as RGBA8 rows.
    ///
    /// Readback is the same path frame comparison will need later, so it copies
    /// the image the compositor receives rather than re-rendering into an
    /// offscreen target.
    pub fn capture_frame(&mut self, colour: [f32; 4]) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, IDENTITY)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    fn render_frame(
        &mut self,
        colour: [f32; 4],
        capture: bool,
        view_projection: [[f32; 4]; 4],
    ) -> Result<Option<Vec<u8>>, VulkanError> {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            for section in std::mem::take(&mut self.retired) {
                self.destroy_section(section);
            }
            self.staging_in_flight.clear();

            let (index, _) = match self.swapchain_device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available,
                vk::Fence::null(),
            ) {
                Ok(value) => value,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.out_of_date = true;
                    return Ok(None);
                }
                Err(result) => return Err(VulkanError::Call("vkAcquireNextImageKHR", result)),
            };
            self.device
                .reset_fences(&[self.in_flight])
                .map_err(|result| VulkanError::Call("vkResetFences", result))?;

            let image = self.images[index as usize];
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1);

            self.device
                .begin_command_buffer(
                    self.command_buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|result| VulkanError::Call("vkBeginCommandBuffer", result))?;
            self.record_staged_copies();

            transition(
                &self.device,
                self.command_buffer,
                image,
                range,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );
            self.device.cmd_clear_color_image(
                self.command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue { float32: colour },
                &[range],
            );

            if self.scene.is_some() {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                );
                self.record_scene(index as usize, view_projection);
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
            }
            if capture {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                );
                let region = vk::BufferImageCopy::default()
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width: self.description.extent.width,
                        height: self.description.extent.height,
                        depth: 1,
                    });
                self.device.cmd_copy_image_to_buffer(
                    self.command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    self.readback_buffer,
                    &[region],
                );
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            } else {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            }

            self.device
                .end_command_buffer(self.command_buffer)
                .map_err(|result| VulkanError::Call("vkEndCommandBuffer", result))?;

            let wait_semaphores = [self.image_available];
            let wait_stages = [vk::PipelineStageFlags::TRANSFER];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished[index as usize]];
            let submit = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            self.device
                .queue_submit(self.queue, &[submit], self.in_flight)
                .map_err(|result| VulkanError::Call("vkQueueSubmit", result))?;

            let swapchains = [self.swapchain];
            let indices = [index];
            let present = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);
            match self.swapchain_device.queue_present(self.queue, &present) {
                Ok(false) => {}
                Ok(true)
                | Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
                | Err(vk::Result::SUBOPTIMAL_KHR) => self.out_of_date = true,
                Err(result) => return Err(VulkanError::Call("vkQueuePresentKHR", result)),
            }

            if !capture {
                return Ok(None);
            }
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let byte_count =
                (self.description.extent.width * self.description.extent.height * 4) as usize;
            let mapped = self
                .device
                .map_memory(
                    self.readback_memory,
                    0,
                    byte_count as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            let mut rgba = vec![0_u8; byte_count];
            std::ptr::copy_nonoverlapping(mapped, rgba.as_mut_ptr(), byte_count);
            self.device.unmap_memory(self.readback_memory);
            if self.description.surface_format == vk::Format::B8G8R8A8_UNORM {
                for pixel in rgba.as_chunks_mut::<4>().0 {
                    pixel.swap(0, 2);
                }
            }
            Ok(Some(rgba))
        }
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.destroy_scene();
            if let Some(sky) = self.sky_buffer.take() {
                self.device.destroy_buffer(sky.buffer, None);
                self.device.free_memory(sky.memory, None);
            }
            if let Some(clouds) = self.cloud_buffer.take() {
                self.device.destroy_buffer(clouds.buffer, None);
                self.device.free_memory(clouds.memory, None);
            }
            if let Some(weather) = self.weather_buffer.take() {
                self.device.destroy_buffer(weather.buffer, None);
                self.device.free_memory(weather.memory, None);
            }
            if let Some(entities) = self.entity_buffer.take() {
                self.device.destroy_buffer(entities.buffer, None);
                self.device.free_memory(entities.memory, None);
            }
            for block in self.arena.drain(..) {
                self.device.destroy_buffer(block.buffer, None);
                self.device.free_memory(block.memory, None);
            }
            for chunk in self.staging.drain(..) {
                self.device.destroy_buffer(chunk.buffer, None);
                self.device.free_memory(chunk.memory, None);
            }
            for view in self.image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            for semaphore in self.render_finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_fence(self.in_flight, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_buffer(self.readback_buffer, None);
            self.device.free_memory(self.readback_memory, None);
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_instance.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

/// Insert the layout transition the clear and the present each need.
unsafe fn transition(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    range: vk::ImageSubresourceRange,
    from: vk::ImageLayout,
    to: vk::ImageLayout,
) {
    unsafe {
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(from)
            .new_layout(to)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(range)
            // Scaffolding barrier: make every prior write available and every
            // later read see it. Once the real pipeline exists these become the
            // specific stage/access pairs each pass needs.
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE);
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }
}

/// Pick a device the way the reference does: skip anything that cannot meet the
/// required extensions or the minimum API version, take the first that can, and
/// upgrade to a discrete GPU if a later one turns out to be discrete.
unsafe fn select_physical_device(
    instance: &ash::Instance,
    surface_instance: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32, vk::PhysicalDeviceProperties), VulkanError> {
    unsafe {
        let devices = instance
            .enumerate_physical_devices()
            .map_err(|result| VulkanError::Call("vkEnumeratePhysicalDevices", result))?;
        let mut selected: Option<(vk::PhysicalDevice, u32, vk::PhysicalDeviceProperties)> = None;

        for device in devices {
            let properties = instance.get_physical_device_properties(device);
            if properties.api_version < MINIMUM_DEVICE_API_VERSION {
                continue;
            }
            let Ok(extensions) = instance.enumerate_device_extension_properties(device) else {
                continue;
            };
            let has_all = REQUIRED_DEVICE_EXTENSIONS.iter().all(|required| {
                extensions
                    .iter()
                    .any(|available| CStr::from_ptr(available.extension_name.as_ptr()) == *required)
            });
            if !has_all {
                continue;
            }
            let Some(queue_family) =
                graphics_present_family(instance, surface_instance, surface, device)
            else {
                continue;
            };

            let discrete = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
            match &selected {
                None => selected = Some((device, queue_family, properties)),
                Some((_, _, chosen)) => {
                    if discrete && chosen.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
                        selected = Some((device, queue_family, properties));
                    }
                }
            }
        }

        selected.ok_or(VulkanError::NoSuitableDevice)
    }
}

/// First queue family that can both render and present to this surface.
unsafe fn graphics_present_family(
    instance: &ash::Instance,
    surface_instance: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    device: vk::PhysicalDevice,
) -> Option<u32> {
    unsafe {
        instance
            .get_physical_device_queue_family_properties(device)
            .iter()
            .enumerate()
            .find_map(|(index, family)| {
                let index = index as u32;
                let graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let present = surface_instance
                    .get_physical_device_surface_support(device, index, surface)
                    .unwrap_or(false);
                (graphics && present).then_some(index)
            })
    }
}

/// The reference accepts only these two formats, and only in sRGB non-linear.
unsafe fn pick_surface_format(
    surface_instance: &ash::khr::surface::Instance,
    device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<vk::Format, VulkanError> {
    unsafe {
        let formats = surface_instance
            .get_physical_device_surface_formats(device, surface)
            .map_err(|result| VulkanError::Call("vkGetPhysicalDeviceSurfaceFormats", result))?;
        formats
            .iter()
            .find(|format| {
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    && ACCEPTED_SURFACE_FORMATS.contains(&format.format)
            })
            .map(|format| format.format)
            .ok_or(VulkanError::NoSuitableSurfaceFormat)
    }
}

/// Keep the requested size inside what the surface reports it can present.
fn clamp_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    width: u32,
    height: u32,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    vk::Extent2D {
        width: width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}
