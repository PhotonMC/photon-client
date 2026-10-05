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

use crate::png::DecodedImage;
use crate::scene::{BlockVertex, Mesh, Vertex};

/// Scaffolding shaders. These are this project's own, not the reference's:
/// they exist to prove the pipeline works, and carry no parity claim.
/// What the terrain's and the entities' vertex shaders share: the push
/// constants, the lightmap, and the vertex inputs each takes.
macro_rules! vertex_prelude {
    ($inputs:expr) => {
        concat!(
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
",
            $inputs,
            r"

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
        )
    };
}

/// The vertex inputs of [`Vertex`], which every pipeline but the terrain's
/// draws with.
macro_rules! vertex_inputs {
    () => {
        r"layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in float in_shade;
layout(location = 3) in vec4 in_tint;
layout(location = 4) in vec2 in_light;
"
    };
}

/// The section meshes' vertex shader: their vertices carry the colour in
/// bytes and the light in whole levels of sixteenths, and no separate shade.
const SECTION_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(
        r"layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_colour;
layout(location = 3) in uvec2 in_light;
"
    ),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out float out_shade;
layout(location = 2) out vec4 out_tint;
layout(location = 3) out vec2 out_fog;
layout(location = 4) out vec3 out_light;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_shade = 1.0;
    out_tint = in_colour;
    out_light = sample_lightmap(vec2(in_light));
    // Fog is measured from the camera: once as a sphere, once as a
    // cylinder whose height counts like its radius.
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

const VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
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

/// The vertex shader of the held and dropped items and of the blocks on
/// entities: [`VERTEX_SHADER`] with the light kept apart from the tint, so the
/// fragment can lay the overlay over the colour before the lightmap.
const HELD_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out float out_shade;
layout(location = 2) out vec4 out_tint;
layout(location = 3) out vec2 out_fog;
layout(location = 4) out vec3 out_light;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_shade = in_shade;
    out_tint = in_tint;
    out_light = sample_lightmap(in_light);
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The entities' vertex shader, after the reference's entity shader with
/// per-face lighting: the vertex's normal rides in its tint, and both faces'
/// light is worked out so the fragment can pick by which way it faces. A zero
/// normal means no per-face lighting and a negative light no lightmap.
const ENTITY_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec3 out_front;
layout(location = 2) out vec3 out_back;
layout(location = 3) out vec3 out_light;
layout(location = 4) out vec2 out_fog;
layout(location = 5) out float out_hurt;
layout(location = 6) out vec3 out_colour;
layout(location = 7) out float out_alpha;
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
    // A vertex with no normal is not shaded by face (`NO_CARDINAL_LIGHTING`),
    // and one with a negative light is not lit by the lightmap (`EMISSIVE`).
    bool shaded = dot(in_tint.xyz, in_tint.xyz) > 0.0;
    out_front = vec3(shaded ? mix_light(light) : 1.0);
    // A negative shade is a pipeline without per-face lighting (`END_CRYSTAL_BEAM`):
    // the one side's light, whichever face shows, and no overlay.
    bool one_sided = in_shade < 0.0;
    out_back = one_sided ? out_front : vec3(shaded ? mix_light(-light) : 1.0);
    out_light = in_light.x < 0.0 ? vec3(1.0) : sample_lightmap(in_light);
    // An emissive vertex whose second light is not negative carries the
    // alpha its colour is blended with (`ARGB.white(alpha)`).
    out_alpha = (in_light.x < 0.0 && in_light.y >= 0.0) ? in_light.y : 1.0;
    // A shade of 64 and over is a blended vertex lit by the world: its alpha in 255ths, no overlay.
    bool blended = in_shade >= 64.0;
    if (blended) { out_alpha = (in_shade - 64.0) / 255.0; }
    // The vertex's shade says whether the hurt overlay is on, and the fourth
    // component of its tint the colour its texel is multiplied by, as
    // 0xRRGGBB (exact in a float), or zero for none.
    out_hurt = (one_sided || blended) ? 0.0 : in_shade;
    float packed_colour = in_tint.w;
    // Zero is no colour (white), a negative one black.
    out_colour = packed_colour == 0.0 ? vec3(1.0) : packed_colour < 0.0 ? vec3(0.0) : vec3(
        mod(floor(packed_colour / 65536.0), 256.0),
        mod(floor(packed_colour / 256.0), 256.0),
        mod(packed_colour, 256.0)) / 255.0;
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The entities' fragment shader: the texture, cut out below a tenth of
/// alpha, lit by whichever face shows, the overlay (the reference's overlay
/// texture: red at 0xB2 of 0xFF alpha when hurt, else white by column), the
/// lightmap, then fog.
const ENTITY_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec3 in_front;
layout(location = 2) in vec3 in_back;
layout(location = 3) in vec3 in_light;
layout(location = 4) in vec2 in_fog;
layout(location = 5) in float in_hurt;
layout(location = 6) in vec3 in_colour;
layout(location = 7) in float in_alpha;
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
    colour.rgb *= in_colour * (gl_FrontFacing ? in_front : in_back);
    // The overlay (`OverlayTexture`): red when hurt, else a white flash by
    // column, which fades the white by `u / 15 * 0.75`.
    if (mod(in_hurt, 2.0) > 0.5) {
        colour.rgb = mix(vec3(1.0, 0.0, 0.0), colour.rgb, 178.0 / 255.0);
    } else if (in_hurt > 1.5) {
        float flash = floor((1.0 - floor(in_hurt / 2.0) / 15.0 * 0.75) * 255.0) / 255.0;
        colour.rgb = mix(vec3(1.0), colour.rgb, flash);
    }
    colour.rgb *= in_light;
    colour.a *= in_alpha;
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

/// The world text's vertex shader, after the reference's text shader: the
/// vertex colour (the tint) lit by the lightmap, and the distances fog reads.
const TEXT_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_colour;
layout(location = 2) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_colour = vec4(in_tint.rgb * sample_lightmap(in_light), in_tint.a);
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The energy swirl's vertex shader (`ENERGY_SWIRL`): no light and no shading;
/// the vertex's tint carries its texture's place in the packed image (the
/// corner and the size), its light the texture offset, and its shade the
/// colour (0xRRGGBB, exact in a float) the texel is multiplied by.
const SWIRL_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) flat out vec4 out_rect;
layout(location = 2) flat out vec2 out_offset;
layout(location = 3) flat out float out_colour;
layout(location = 4) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_rect = in_tint;
    out_offset = in_light;
    out_colour = in_shade;
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The energy swirl's fragment shader: the texture moved by the offset and
/// wrapped inside its own place in the packed image (the reference's texture
/// repeats), cut out below a tenth of alpha, times its colour, then fog.
const SWIRL_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) flat in vec4 in_rect;
layout(location = 2) flat in vec2 in_offset;
layout(location = 3) flat in float in_colour;
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
    vec2 local = (in_uv - in_rect.xy) / in_rect.zw + in_offset;
    vec2 wrapped = in_rect.xy + fract(local) * in_rect.zw;
    vec4 colour = texture(skin, wrapped);
    if (colour.a < 0.1) { discard; }
    colour.rgb *= vec3(
        mod(floor(in_colour / 65536.0), 256.0),
        mod(floor(in_colour / 256.0), 256.0),
        mod(in_colour, 256.0)) / 255.0;
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

/// The dissolving entity's fragment shader (`ENTITY_CUTOUT_DISSOLVE`): the entity's, with
/// no overlay, and each texel dropped where the vertex's alpha (the shade, which
/// carries it) is below the mask's. The mask is another texture of the packed
/// image, at the same place in it moved by the push constant's offset.
const DISSOLVE_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec3 in_front;
layout(location = 2) in vec3 in_back;
layout(location = 3) in vec3 in_light;
layout(location = 4) in vec2 in_fog;
layout(location = 5) in float in_alpha_mask;
layout(location = 6) in vec3 in_colour;
layout(location = 7) in float in_unused;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
    vec4 dissolve;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
void main() {
    vec4 colour = texture(skin, in_uv);
    if (colour.a < 0.1) { discard; }
    if (in_alpha_mask < texture(skin, in_uv + push.dissolve.xy).a) { discard; }
    colour.rgb *= in_colour * (gl_FrontFacing ? in_front : in_back);
    colour.rgb *= in_light;
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

/// The dying dragon's rays' vertex shader (`DRAGON_RAYS`, the lightning's shaders): the
/// colour as it is, and the distances fog reads.
const RAYS_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"layout(location = 0) out vec4 out_colour;
layout(location = 1) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_colour = in_tint;
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The rays' fragment shader: the colour, less what fog takes of it.
const RAYS_FRAGMENT_SHADER: &str = r"#version 450
layout(location = 0) in vec4 in_colour;
layout(location = 1) in vec2 in_fog;
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
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = in_colour * (1.0 - fog);
}
";

/// The shadow's vertex shader (`rendertype_entity_shadow`): the colour as it
/// is, with no light, and the distances fog reads.
const SHADOW_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_colour;
layout(location = 2) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_colour = in_tint;
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The shadow's fragment shader: the texel times the colour, no cut-out,
/// then fog.
const SHADOW_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec2 in_fog;
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
    vec4 colour = texture(skin, in_uv) * in_colour;
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

/// The same text with `IS_SEE_THROUGH`: the colour as it is, no fog.
const TEXT_SEE_THROUGH_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_colour;
layout(location = 2) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_colour = in_tint;
    out_fog = vec2(0.0);
}
",
);

/// The world text's fragment shader: the glyph's texel times the colour, cut
/// out below a tenth of alpha, then fog. A vertex with a negative texture
/// coordinate has no texel (the name tag's background box).
const TEXT_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D font;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec2 in_fog;
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
    vec4 texel = in_uv.x < 0.0 ? vec4(1.0) : texture(font, in_uv);
    vec4 colour = texel * in_colour;
    if (colour.a < 0.1) { discard; }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

/// The see-through text's fragment shader: no fog.
const TEXT_SEE_THROUGH_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D font;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
void main() {
    vec4 texel = in_uv.x < 0.0 ? vec4(1.0) : texture(font, in_uv);
    vec4 colour = texel * in_colour;
    if (colour.a < 0.1) { discard; }
    out_colour = colour;
}
";

/// The block outline's shaders, after the reference's line shader: each
/// vertex is pushed sideways on screen by half its signed width, across the
/// edge's direction as the screen sees it, so a line keeps its width in
/// pixels at any distance. The colour then fogs like the terrain.
const LINE_VERTEX_SHADER: &str = r"#version 450
layout(push_constant) uniform Push {
    mat4 view_projection;
    // The framebuffer's size in pixels.
    vec4 screen;
    vec4 unused_block;
    vec4 unused_ambient;
    vec4 camera;
    vec4 fog_colour;
    vec4 fog_distances;
} push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec3 in_normal;
layout(location = 3) in float in_width;
layout(location = 0) out vec4 out_colour;
layout(location = 1) out vec2 out_fog;
void main() {
    vec4 start = push.view_projection * vec4(in_position, 1.0);
    vec4 end = push.view_projection * vec4(in_position + in_normal, 1.0);
    vec3 ndc_start = start.xyz / start.w;
    vec3 ndc_end = end.xyz / end.w;
    vec2 screen = push.screen.xy;
    vec2 direction = normalize((ndc_end.xy - ndc_start.xy) * screen);
    vec2 offset = vec2(-direction.y, direction.x) * abs(in_width) / screen;
    if (offset.x < 0.0) {
        offset = -offset;
    }
    offset *= sign(in_width);
    gl_Position = vec4((ndc_start + vec3(offset, 0.0)) * start.w, start.w);
    out_colour = in_colour;
    vec3 from_camera = in_position - push.camera.xyz;
    out_fog = vec2(length(from_camera), max(length(from_camera.xz), abs(from_camera.y)));
}
";

const LINE_FRAGMENT_SHADER: &str = r"#version 450
layout(location = 0) in vec4 in_colour;
layout(location = 1) in vec2 in_fog;
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
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(in_colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), in_colour.a);
}
";

/// The breaking overlay's fragment shader, after the reference's crumbling
/// shader: the stage's crack, cut out below a tenth of alpha, then fog. The
/// stages are stacked tiles; the vertex's shade names the tile and the
/// coordinates wrap within it.
const CRUMBLING_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D cracks;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in float in_shade;
layout(location = 2) in vec4 in_tint;
layout(location = 3) in vec2 in_fog;
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
    vec2 tile = vec2(fract(in_uv.x), (in_shade + fract(in_uv.y)) / 10.0);
    vec4 colour = texture(cracks, tile);
    if (colour.a < 0.1) { discard; }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
";

/// The GUI's shaders, after the reference's textured GUI shader: a sprite
/// texel times the vertex colour, dropped where it is fully clear. Vertices
/// are in GUI units, mapped onto the window by the push constant.
const GUI_VERTEX_SHADER: &str = r"#version 450
layout(push_constant) uniform Push {
    // Two over the window's size in GUI units.
    vec4 scale;
} push;
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_colour;
layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_colour;
void main() {
    gl_Position = vec4(in_position * push.scale.xy - 1.0, 0.0, 1.0);
    out_uv = in_uv;
    out_colour = in_colour;
}
";

const GUI_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D sprites;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 0) out vec4 out_colour;
void main() {
    vec4 colour = texture(sprites, in_uv) * in_colour;
    if (colour.a == 0.0) { discard; }
    out_colour = colour;
}
";

/// The shaders for items in the GUI, after the reference's item shader as it
/// draws into the GUI item atlas: the atlas texel at its nearest, cut out
/// below a tenth of alpha, times the vertex colour, which already carries the
/// tint and the item light. Depth runs towards the viewer across the
/// orthographic range of a thousand units either way.
const GUI_ITEM_VERTEX_SHADER: &str = r"#version 450
layout(push_constant) uniform Push {
    vec4 scale;
} push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_colour;
layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_colour;
void main() {
    gl_Position = vec4(in_position.xy * push.scale.xy - 1.0, 0.5 + in_position.z / 2000.0, 1.0);
    out_uv = in_uv;
    out_colour = in_colour;
}
";

const GUI_ITEM_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 0) out vec4 out_colour;
void main() {
    ivec2 size = textureSize(atlas, 0);
    vec4 texel = texelFetch(atlas, clamp(ivec2(floor(in_uv * vec2(size))), ivec2(0), size - 1), 0);
    if (texel.a < 0.1) { discard; }
    out_colour = texel * in_colour;
}
";

const FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in float in_shade;
layout(location = 2) in vec4 in_tint;
layout(location = 3) in vec2 in_fog;
layout(location = 4) in vec3 in_light;
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
    // The shade carries the entity overlay (`OverlayTexture`) above two: the
    // shade itself is below two, so the whole part of half of it is the
    // overlay's code, as the entities' shader reads it.
    float overlay = floor(in_shade / 2.0);
    texel.rgb *= in_shade - overlay * 2.0;
    if (mod(overlay, 2.0) > 0.5) {
        texel.rgb = mix(vec3(1.0, 0.0, 0.0), texel.rgb, 178.0 / 255.0);
    } else if (overlay > 1.5) {
        float flash = floor((1.0 - floor(overlay / 2.0) / 15.0 * 0.75) * 255.0) / 255.0;
        texel.rgb = mix(vec3(1.0), texel.rgb, flash);
    }
    texel.rgb *= in_light;
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
layout(location = 4) in vec3 in_light;
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
    size: u64,
    /// Free ranges, offset and length, in offset order. A block given back
    /// to the driver has a null buffer and none.
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

/// The sky's vertex buffer, rewritten every frame.
struct SkyBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    capacity: usize,
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
    command_buffer: vk::CommandBuffer,
    /// The buffer a screenshot is copied into, there only while one is taken.
    readback: Option<(vk::Buffer, vk::DeviceMemory)>,
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
    /// The dropped items' triangles, which sample the block atlas.
    ground_buffer: Option<SkyBuffer>,
    ground_count: u32,
    /// The name tags' text: seen through blocks, then lit and depth-tested.
    name_tag_see_buffer: Option<SkyBuffer>,
    name_tag_see_count: u32,
    /// The experience orbs' triangles, which sample the entity textures.
    orb_buffer: Option<SkyBuffer>,
    orb_count: u32,
    /// The entity shadows' triangles, which sample the entity textures.
    shadow_buffer: Option<SkyBuffer>,
    shadow_count: u32,
    /// The glowing layers' triangles (`RenderTypes.eyes`): the entity
    /// textures, unlit and blended, tested but not written to the depth.
    glow_buffer: Option<SkyBuffer>,
    glow_count: u32,
    /// The energy swirls' triangles (`RenderTypes.energySwirl`).
    swirl_buffer: Option<SkyBuffer>,
    swirl_count: u32,
    /// What the dissolving entities draw (a dying dragon's body), and where
    /// each texel's mask is from it in the packed image (in texture units).
    dissolve_buffer: Option<SkyBuffer>,
    dissolve_count: u32,
    dissolve_mask_offset: [f32; 2],
    /// The dying dragon's rays.
    rays_buffer: Option<SkyBuffer>,
    rays_count: u32,
    name_tag_buffer: Option<SkyBuffer>,
    name_tag_count: u32,
    /// The first-person hand's triangles, in view space, and the projection
    /// they are drawn with over a cleared depth.
    hand_buffer: Option<SkyBuffer>,
    hand_count: u32,
    /// The held items' triangles, after the arm's in the hand buffer.
    held_count: u32,
    /// The aimed block's outline strips for the next frame.
    outline_buffer: Option<SkyBuffer>,
    outline_count: u32,
    /// The breaking overlay's triangles for the next frame.
    breaking_buffer: Option<SkyBuffer>,
    breaking_count: u32,
    hand_projection: [[f32; 4]; 4],
    /// The GUI's triangles, their runs by blend, and the window's size in
    /// GUI units.
    gui_buffer: Option<SkyBuffer>,
    gui_batches: Vec<(crate::gui::GuiBlend, u32, u32)>,
    gui_size: [f32; 2],
    /// The items in the GUI's slots, which the GUI's item runs count in.
    gui_item_buffer: Option<SkyBuffer>,
    sections: std::collections::BTreeMap<(i32, i32, i32), SectionBuffers>,
    /// The sections in view this frame; `None` draws every section.
    visible_sections: Option<Vec<(i32, i32, i32)>>,
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
    #[expect(
        clippy::too_many_lines,
        reason = "Vulkan objects created in the order the API requires"
    )]
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

            // Layered drivers such as MoltenVK register as portability drivers,
            // which the loader hides unless the instance opts in. Opting in only
            // widens enumeration; conformant drivers are listed either way.
            let portability = entry
                .enumerate_instance_extension_properties(None)
                .map_err(|result| VulkanError::Call("enumerate instance extensions", result))?
                .iter()
                .any(|available| {
                    CStr::from_ptr(available.extension_name.as_ptr())
                        == ash::khr::portability_enumeration::NAME
                });
            let mut instance_flags = vk::InstanceCreateFlags::empty();
            if portability {
                extensions.push(ash::khr::portability_enumeration::NAME.as_ptr());
                instance_flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
            }

            let application_name = CString::new("Minecraft Java Edition").expect("static name");
            let engine_name = CString::new("MinecraftJE").expect("static name");
            let application_info = vk::ApplicationInfo::default()
                .application_name(&application_name)
                .engine_name(&engine_name)
                .engine_version(0)
                .api_version(API_VERSION);
            let instance_info = vk::InstanceCreateInfo::default()
                .flags(instance_flags)
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
            let mut extension_names: Vec<*const i8> = REQUIRED_DEVICE_EXTENSIONS
                .iter()
                .map(|name| name.as_ptr())
                .collect();
            // A portability driver's device must be created with this extension
            // enabled whenever it advertises it.
            let portability_subset = instance
                .enumerate_device_extension_properties(physical_device)
                .map_err(|result| VulkanError::Call("enumerate device extensions", result))?
                .iter()
                .any(|available| {
                    CStr::from_ptr(available.extension_name.as_ptr())
                        == ash::khr::portability_subset::NAME
                });
            if portability_subset {
                extension_names.push(ash::khr::portability_subset::NAME.as_ptr());
            }

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
                readback: None,
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
                ground_buffer: None,
                ground_count: 0,
                name_tag_see_buffer: None,
                name_tag_see_count: 0,
                orb_buffer: None,
                orb_count: 0,
                shadow_buffer: None,
                shadow_count: 0,
                glow_buffer: None,
                glow_count: 0,
                swirl_buffer: None,
                swirl_count: 0,
                dissolve_buffer: None,
                dissolve_count: 0,
                dissolve_mask_offset: [0.0; 2],
                rays_buffer: None,
                rays_count: 0,
                name_tag_buffer: None,
                name_tag_count: 0,
                hand_buffer: None,
                hand_count: 0,
                held_count: 0,
                outline_buffer: None,
                outline_count: 0,
                breaking_buffer: None,
                breaking_count: 0,
                hand_projection: [[0.0; 4]; 4],
                gui_buffer: None,
                gui_batches: Vec::new(),
                gui_size: [1.0, 1.0],
                gui_item_buffer: None,
                sections: std::collections::BTreeMap::new(),
                visible_sections: None,
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
    /// The buffer a screenshot of the current extent is copied into, made
    /// when one is taken and freed once it has been read.
    unsafe fn make_readback(&mut self) -> Result<(), VulkanError> {
        unsafe {
            self.free_readback();
            let extent = self.description.extent;
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(u64::from(extent.width) * u64::from(extent.height) * 4)
                        .usage(vk::BufferUsageFlags::TRANSFER_DST)
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
            self.readback = Some((buffer, memory));
            Ok(())
        }
    }

    unsafe fn free_readback(&mut self) {
        unsafe {
            if let Some((buffer, memory)) = self.readback.take() {
                self.device.destroy_buffer(buffer, None);
                self.device.free_memory(memory, None);
            }
        }
    }

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

    /// The shadows, experience orbs and name tags: flat shapes drawn after the
    /// entities, each from its own buffer.
    unsafe fn record_flat_shapes(&self, scene: &SceneResources, terrain_push: &[f32]) {
        unsafe {
            // Shadows under the entities, before them: blended quads of the
            // shadow texture over the blocks' tops.
            if let (Some(buffer), Some(set)) = (self.shadow_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    set,
                    (buffer, self.shadow_count),
                    terrain_push,
                );
            }
            // Dissolving entities (a dying dragon's body), over the entities'
            // own draw: its push constants carry where the masks are.
            if let (Some(buffer), Some(set)) = (self.dissolve_buffer.as_ref(), scene.entity_set)
                && self.dissolve_count > 0
            {
                let mut push = [0.0_f32; 44];
                push[..40].copy_from_slice(&terrain_push[..40]);
                push[40] = self.dissolve_mask_offset[0];
                push[41] = self.dissolve_mask_offset[1];
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.dissolve_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.dissolve_pipeline_layout,
                    0,
                    &[set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.dissolve_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(push.as_ptr().cast::<u8>(), 176),
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.dissolve_count, 1, 0, 0);
            }
            // The glowing layers (the eyes of spiders and endermen): the
            // `EYES` pipeline is the shadow's shaders and state, the texel
            // times the colour with no light and no cut-out, then fog.
            if let (Some(buffer), Some(set)) = (self.glow_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    set,
                    (buffer, self.glow_count),
                    terrain_push,
                );
            }
            // The energy swirls (a charged creeper's): added over the entities.
            if let (Some(buffer), Some(set)) = (self.swirl_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.swirl_pipeline, scene.swirl_pipeline_layout),
                    set,
                    (buffer, self.swirl_count),
                    terrain_push,
                );
            }
            // A dying dragon's rays: added by their alpha over what is behind.
            if let (Some(buffer), Some(set)) = (self.rays_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (scene.rays_pipeline, scene.rays_pipeline_layout),
                    set,
                    (buffer, self.rays_count),
                    terrain_push,
                );
            }
            // Experience orbs: blended quads over the entity textures.
            if let (Some(buffer), Some(set)) = (self.orb_buffer.as_ref(), scene.entity_set) {
                self.draw_triangles(
                    (
                        scene.text_translucent_pipeline,
                        scene.text_translucent_pipeline_layout,
                    ),
                    set,
                    (buffer, self.orb_count),
                    terrain_push,
                );
            }
            // Name tags after the entities: the text seen through blocks
            // first, then the text tested against the depth.
            if let Some(set) = scene.gui_set {
                for (buffer, count, pipeline) in [
                    (
                        self.name_tag_see_buffer.as_ref(),
                        self.name_tag_see_count,
                        (
                            scene.text_see_through_pipeline,
                            scene.text_see_through_pipeline_layout,
                        ),
                    ),
                    (
                        self.name_tag_buffer.as_ref(),
                        self.name_tag_count,
                        (scene.text_pipeline, scene.text_pipeline_layout),
                    ),
                ] {
                    if let Some(buffer) = buffer {
                        self.draw_triangles(pipeline, set, (buffer, count), terrain_push);
                    }
                }
            }
        }
    }

    /// Draw a buffer's triangles with a pipeline over a descriptor set and the
    /// frame's push constants. Nothing is drawn for an empty buffer.
    unsafe fn draw_triangles(
        &self,
        (pipeline, layout): (vk::Pipeline, vk::PipelineLayout),
        set: vk::DescriptorSet,
        (buffer, count): (&SkyBuffer, u32),
        push: &[f32],
    ) {
        if count == 0 {
            return;
        }
        unsafe {
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
                &[set],
                &[],
            );
            self.device.cmd_push_constants(
                self.command_buffer,
                layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                std::slice::from_raw_parts(push.as_ptr().cast::<u8>(), 160),
            );
            self.device
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
            self.device.cmd_draw(self.command_buffer, count, 1, 0, 0);
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one command buffer recorded in draw order"
    )]
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
                        crate::sky::SkyBlend::Translucent => {
                            (scene.sky_pipeline_layout, scene.sky_pipeline)
                        }
                        crate::sky::SkyBlend::Overlay => (
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
                .drawn_sections()
                .into_iter()
                .filter(|section| section.index_count > 0)
            {
                self.bind_section(section, scene.pipeline_layout, now);
                self.device
                    .cmd_draw_indexed(self.command_buffer, section.index_count, 1, 0, 0, 0);
            }
            // Dropped items and the blocks on entities, opaque, before the
            // entities so that a translucent shell (a sulfur cube's) blends
            // over the block inside it: the items' quads in the block atlas,
            // shaded as the held ones are, fully faded in.
            if let Some(buffer) = self.ground_buffer.as_ref()
                && self.ground_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.held_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.held_pipeline_layout,
                    0,
                    &[scene.descriptor_set],
                    &[],
                );
                let mut ground_push = [0.0_f32; 41];
                ground_push[..40].copy_from_slice(&terrain_push);
                ground_push[40] = 1.0;
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.held_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(ground_push.as_ptr().cast::<u8>(), 164),
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.ground_count, 1, 0, 0);
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
            self.record_flat_shapes(scene, &terrain_push);
            // The aimed block's outline over the solid world.
            if let Some(buffer) = self.outline_buffer.as_ref()
                && self.outline_count > 0
            {
                let mut line_push = terrain_push;
                line_push[16] = extent.width as f32;
                line_push[17] = extent.height as f32;
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.lines_pipeline,
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.lines_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(line_push.as_ptr().cast::<u8>(), 160),
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.outline_count, 1, 0, 0);
            }
            // Cracks over the blocks being broken.
            if let (Some(buffer), Some(set)) = (self.breaking_buffer.as_ref(), scene.crack_set)
                && self.breaking_count > 0
            {
                self.device.cmd_bind_pipeline(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.crumbling_pipeline,
                );
                self.device.cmd_bind_descriptor_sets(
                    self.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    scene.crumbling_pipeline_layout,
                    0,
                    &[set],
                    &[],
                );
                self.device.cmd_push_constants(
                    self.command_buffer,
                    scene.crumbling_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(terrain_push.as_ptr().cast::<u8>(), 160),
                );
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                self.device
                    .cmd_draw(self.command_buffer, self.breaking_count, 1, 0, 0);
            }
            // Translucent faces last, the farthest section first, so nearer
            // water blends over farther.
            let mut translucent: Vec<&SectionBuffers> = self
                .drawn_sections()
                .into_iter()
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
            // The first-person hand last, over a cleared depth, with its own
            // projection and the camera at the origin of its view space.
            if let Some(buffer) = self.hand_buffer.as_ref()
                && self.hand_count + self.held_count > 0
            {
                self.device.cmd_clear_attachments(
                    self.command_buffer,
                    &[vk::ClearAttachment {
                        aspect_mask: vk::ImageAspectFlags::DEPTH,
                        color_attachment: 0,
                        clear_value: vk::ClearValue {
                            depth_stencil: vk::ClearDepthStencilValue {
                                depth: 0.0,
                                stencil: 0,
                            },
                        },
                    }],
                    &[vk::ClearRect {
                        rect: vk::Rect2D::default().extent(self.description.extent),
                        base_array_layer: 0,
                        layer_count: 1,
                    }],
                );
                let mut hand_push = terrain_push;
                hand_push[..16].copy_from_slice(&std::array::from_fn::<f32, 16, _>(|i| {
                    self.hand_projection[i / 4][i % 4]
                }));
                hand_push[28..31].copy_from_slice(&[0.0; 3]);
                self.device
                    .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
                if let Some(set) = scene.entity_set
                    && self.hand_count > 0
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
                        std::slice::from_raw_parts(hand_push.as_ptr().cast::<u8>(), 160),
                    );
                    self.device
                        .cmd_draw(self.command_buffer, self.hand_count, 1, 0, 0);
                }
                // The held items after the arm, from the block atlas the way
                // the terrain samples it, fully faded in.
                if self.held_count > 0 {
                    self.device.cmd_bind_pipeline(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.held_pipeline,
                    );
                    self.device.cmd_bind_descriptor_sets(
                        self.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        scene.held_pipeline_layout,
                        0,
                        &[scene.descriptor_set],
                        &[],
                    );
                    let mut held_push = [0.0_f32; 41];
                    held_push[..40].copy_from_slice(&hand_push);
                    held_push[40] = 1.0;
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        scene.held_pipeline_layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        std::slice::from_raw_parts(held_push.as_ptr().cast::<u8>(), 164),
                    );
                    self.device.cmd_draw(
                        self.command_buffer,
                        self.held_count,
                        1,
                        self.hand_count,
                        0,
                    );
                }
            }
            // The GUI over everything, run by run in its blends.
            if let (Some(buffer), Some(set)) = (self.gui_buffer.as_ref(), scene.gui_set)
                && !self.gui_batches.is_empty()
            {
                let push = [2.0 / self.gui_size[0], 2.0 / self.gui_size[1], 0.0, 0.0];
                for &(blend, first, count) in &self.gui_batches {
                    let (pipeline, layout, set, vertices) = match blend {
                        crate::gui::GuiBlend::Translucent => {
                            (scene.gui_pipeline, scene.gui_pipeline_layout, set, buffer)
                        }
                        crate::gui::GuiBlend::Invert => (
                            scene.gui_invert_pipeline,
                            scene.gui_invert_pipeline_layout,
                            set,
                            buffer,
                        ),
                        crate::gui::GuiBlend::Items | crate::gui::GuiBlend::Entity => {
                            let Some(items) = self.gui_item_buffer.as_ref() else {
                                continue;
                            };
                            // The player's model samples the skin; the items
                            // sample the block atlas.
                            let samples = if blend == crate::gui::GuiBlend::Entity {
                                let Some(skin) = scene.entity_set else {
                                    continue;
                                };
                                skin
                            } else {
                                scene.descriptor_set
                            };
                            // Each item is drawn over its own cleared depth
                            // in the reference's item atlas; the slots never
                            // overlap, so one clear serves them all.
                            self.device.cmd_clear_attachments(
                                self.command_buffer,
                                &[vk::ClearAttachment {
                                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                                    color_attachment: 0,
                                    clear_value: vk::ClearValue {
                                        depth_stencil: vk::ClearDepthStencilValue {
                                            depth: 0.0,
                                            stencil: 0,
                                        },
                                    },
                                }],
                                &[vk::ClearRect {
                                    rect: vk::Rect2D::default().extent(self.description.extent),
                                    base_array_layer: 0,
                                    layer_count: 1,
                                }],
                            );
                            (
                                scene.gui_item_pipeline,
                                scene.gui_item_pipeline_layout,
                                samples,
                                items,
                            )
                        }
                    };
                    self.device.cmd_bind_vertex_buffers(
                        self.command_buffer,
                        0,
                        &[vertices.buffer],
                        &[0],
                    );
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
                        &[set],
                        &[],
                    );
                    self.device.cmd_push_constants(
                        self.command_buffer,
                        layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        std::slice::from_raw_parts(push.as_ptr().cast::<u8>(), 16),
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
    #[expect(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "Vulkan objects created in the order the API requires"
    )]
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
        let dissolve = kind == PipelineKind::Dissolve;
        let rays = kind == PipelineKind::Rays;
        let entity = kind == PipelineKind::Entity || dissolve;
        let shadow = kind == PipelineKind::Shadow;
        let swirl = kind == PipelineKind::Swirl;
        let text = matches!(kind, PipelineKind::Text | PipelineKind::TextTranslucent) || shadow;
        let text_see_through = kind == PipelineKind::TextSeeThrough;
        let gui = matches!(kind, PipelineKind::Gui | PipelineKind::GuiInvert);
        let gui_item = kind == PipelineKind::GuiItem;
        let lines = kind == PipelineKind::Lines;
        let crumbling = kind == PipelineKind::Crumbling;
        let section = matches!(kind, PipelineKind::Terrain | PipelineKind::Translucent);
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
            } else if dissolve {
                (ENTITY_VERTEX_SHADER, DISSOLVE_FRAGMENT_SHADER)
            } else if rays {
                (RAYS_VERTEX_SHADER, RAYS_FRAGMENT_SHADER)
            } else if entity {
                (ENTITY_VERTEX_SHADER, ENTITY_FRAGMENT_SHADER)
            } else if swirl {
                (SWIRL_VERTEX_SHADER, SWIRL_FRAGMENT_SHADER)
            } else if shadow {
                (SHADOW_VERTEX_SHADER, SHADOW_FRAGMENT_SHADER)
            } else if text {
                (TEXT_VERTEX_SHADER, TEXT_FRAGMENT_SHADER)
            } else if text_see_through {
                (
                    TEXT_SEE_THROUGH_VERTEX_SHADER,
                    TEXT_SEE_THROUGH_FRAGMENT_SHADER,
                )
            } else if gui {
                (GUI_VERTEX_SHADER, GUI_FRAGMENT_SHADER)
            } else if gui_item {
                (GUI_ITEM_VERTEX_SHADER, GUI_ITEM_FRAGMENT_SHADER)
            } else if lines {
                (LINE_VERTEX_SHADER, LINE_FRAGMENT_SHADER)
            } else if crumbling {
                (VERTEX_SHADER, CRUMBLING_FRAGMENT_SHADER)
            } else if section {
                (SECTION_VERTEX_SHADER, FRAGMENT_SHADER)
            } else {
                (HELD_VERTEX_SHADER, FRAGMENT_SHADER)
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
                std::mem::size_of::<crate::sky::SkyVertex>()
            } else if clouds {
                std::mem::size_of::<crate::clouds::CloudVertex>()
            } else if gui {
                std::mem::size_of::<crate::gui::GuiVertex>()
            } else if gui_item {
                std::mem::size_of::<crate::gui::items::GuiItemVertex>()
            } else if lines {
                std::mem::size_of::<crate::outline::LineVertex>()
            } else if section {
                std::mem::size_of::<BlockVertex>()
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
            let section_attributes = [
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
                    .format(vk::Format::R8G8B8A8_UNORM)
                    .offset(20),
                vk::VertexInputAttributeDescription::default()
                    .location(3)
                    .binding(0)
                    .format(vk::Format::R8G8_UINT)
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
            let gui_attributes = [
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(8),
                vk::VertexInputAttributeDescription::default()
                    .location(2)
                    .binding(0)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(16),
            ];
            let line_attributes = [
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(12),
                vk::VertexInputAttributeDescription::default()
                    .location(2)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(28),
                vk::VertexInputAttributeDescription::default()
                    .location(3)
                    .binding(0)
                    .format(vk::Format::R32_SFLOAT)
                    .offset(40),
            ];
            let gui_item_attributes = [
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
            ];
            let attributes: &[vk::VertexInputAttributeDescription] = if sky {
                &sky_attributes
            } else if gui_item {
                &gui_item_attributes
            } else if clouds {
                &cloud_attributes
            } else if gui {
                &gui_attributes
            } else if lines {
                &line_attributes
            } else if section {
                &section_attributes
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
                .cull_mode(
                    if sky
                        || weather
                        || entity
                        || swirl
                        || rays
                        || text
                        || text_see_through
                        || gui
                        || gui_item
                        || lines
                    {
                        vk::CullModeFlags::NONE
                    } else {
                        vk::CullModeFlags::BACK
                    },
                )
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .line_width(1.0)
                // The crack is pulled towards the camera so it wins over the
                // block's own faces: the reference's polygon offset.
                .depth_bias_enable(crumbling)
                .depth_bias_constant_factor(if crumbling { 10.0 } else { 0.0 })
                .depth_bias_slope_factor(if crumbling { 1.0 } else { 0.0 });
            let multisample = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let depth_state = vk::PipelineDepthStencilStateCreateInfo::default()
                // The sky is behind everything and tests nothing. Translucent
                // faces are tested against the opaque depth but leave it
                // alone, so what is behind them still shows. Depth is
                // reversed and equal depth passes, as in the reference, so
                // a model's later face over an earlier one at the same place
                // (the grass side overlay) is the one seen.
                .depth_test_enable(!sky && !gui && !text_see_through)
                .depth_write_enable(matches!(
                    kind,
                    PipelineKind::Terrain
                        | PipelineKind::Clouds
                        | PipelineKind::Entity
                        | PipelineKind::Dissolve
                        | PipelineKind::Rays
                        | PipelineKind::Swirl
                        | PipelineKind::Text
                        | PipelineKind::Lines
                        | PipelineKind::GuiItem
                ))
                .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
            let blends = [if crumbling {
                // Source times destination, twice: darkens and lightens.
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::DST_COLOR)
                    .dst_color_blend_factor(vk::BlendFactor::SRC_COLOR)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if kind == PipelineKind::GuiInvert {
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::ONE_MINUS_DST_COLOR)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_COLOR)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if kind == PipelineKind::SkyOverlay || rays {
                // Added by its alpha (`BlendFunction.LIGHTNING`).
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if swirl {
                // Added to what is behind (`BlendFunction.ADDITIVE`).
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::ONE)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if translucent
                || sky
                || clouds
                || weather
                || entity
                || text
                || text_see_through
                || gui
                || gui_item
                || lines
            {
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
                        .size(176)
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

    /// Draw only these sections from now on (the ones in view), or every
    /// section for `None`.
    pub fn set_visible_sections(&mut self, visible: Option<&[(i32, i32, i32)]>) {
        self.visible_sections = visible.map(<[_]>::to_vec);
    }

    /// The sections drawn this frame: the ones in view, or all of them.
    fn drawn_sections(&self) -> Vec<&SectionBuffers> {
        match &self.visible_sections {
            Some(visible) => visible
                .iter()
                .filter_map(|key| self.sections.get(key))
                .collect(),
            None => self.sections.values().collect(),
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
        unsafe { self.trim_arena() };
    }

    /// Give every block that holds nothing back to the driver but one, which
    /// stays for the next meshes.
    unsafe fn trim_arena(&mut self) {
        unsafe {
            let mut spare = false;
            for block in &mut self.arena {
                let empty = block.buffer != vk::Buffer::null()
                    && block.free.as_slice() == [(0, block.size)];
                if !empty {
                    continue;
                }
                if !spare {
                    spare = true;
                    continue;
                }
                self.device.destroy_buffer(block.buffer, None);
                self.device.free_memory(block.memory, None);
                block.buffer = vk::Buffer::null();
                block.memory = vk::DeviceMemory::null();
                block.free = Vec::new();
            }
        }
    }

    /// Give the staging chunks past the spare ones back to the driver; none
    /// is in use when a frame starts.
    unsafe fn trim_staging(&mut self) {
        unsafe {
            let mut idle = 0;
            for (index, chunk) in self.staging.iter_mut().enumerate() {
                if chunk.buffer == vk::Buffer::null() || self.staging_filling.contains(&index) {
                    continue;
                }
                idle += 1;
                if idle <= STAGING_SPARE {
                    continue;
                }
                self.device.unmap_memory(chunk.memory);
                self.device.destroy_buffer(chunk.buffer, None);
                self.device.free_memory(chunk.memory, None);
                chunk.buffer = vk::Buffer::null();
                chunk.memory = vk::DeviceMemory::null();
                chunk.mapped = std::ptr::null_mut();
                chunk.size = 0;
                chunk.used = 0;
            }
        }
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
                    let block = self.create_arena_block(size.max(ARENA_BLOCK_SIZE))?;
                    let given_back = self
                        .arena
                        .iter()
                        .position(|own| own.buffer == vk::Buffer::null());
                    match given_back {
                        Some(slot) => {
                            self.arena[slot] = block;
                            (slot, 0)
                        }
                        None => {
                            self.arena.push(block);
                            (self.arena.len() - 1, 0)
                        }
                    }
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
                            let made = self.create_staging_chunk(size.max(STAGING_CHUNK_SIZE))?;
                            let given_back = self
                                .staging
                                .iter()
                                .position(|own| own.buffer == vk::Buffer::null());
                            match given_back {
                                Some(slot) => {
                                    self.staging[slot] = made;
                                    slot
                                }
                                None => {
                                    self.staging.push(made);
                                    self.staging.len() - 1
                                }
                            }
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
                size,
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
    #[expect(
        clippy::too_many_lines,
        reason = "Vulkan objects created in the order the API requires"
    )]
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
            // The atlas's set, the rain's and the snow's, the skin's, the
            // GUI's and the cracks'.
            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(6)];
            let descriptor_pool = self
                .device
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo::default()
                        .max_sets(6)
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
            let (gui_pipeline_layout, gui_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Gui)?;
            let (lines_pipeline_layout, lines_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Lines)?;
            let (crumbling_pipeline_layout, crumbling_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Crumbling)?;
            let (held_pipeline_layout, held_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Held)?;
            let (text_pipeline_layout, text_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Text)?;
            let (text_see_through_pipeline_layout, text_see_through_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::TextSeeThrough)?;
            let (text_translucent_pipeline_layout, text_translucent_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::TextTranslucent)?;
            let (shadow_pipeline_layout, shadow_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Shadow)?;
            let (swirl_pipeline_layout, swirl_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Swirl)?;
            let (dissolve_pipeline_layout, dissolve_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Dissolve)?;
            let (rays_pipeline_layout, rays_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::Rays)?;
            let (gui_invert_pipeline_layout, gui_invert_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::GuiInvert)?;
            let (gui_item_pipeline_layout, gui_item_pipeline) =
                self.create_pipeline(descriptor_layout, PipelineKind::GuiItem)?;

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
                lines_pipeline_layout,
                lines_pipeline,
                crumbling_pipeline_layout,
                crumbling_pipeline,
                held_pipeline_layout,
                held_pipeline,
                text_pipeline_layout,
                text_pipeline,
                text_see_through_pipeline_layout,
                text_see_through_pipeline,
                text_translucent_pipeline_layout,
                text_translucent_pipeline,
                shadow_pipeline_layout,
                shadow_pipeline,
                swirl_pipeline_layout,
                swirl_pipeline,
                dissolve_pipeline_layout,
                dissolve_pipeline,
                rays_pipeline_layout,
                rays_pipeline,
                crack_texture: None,
                crack_sampler: None,
                crack_set: None,
                entity_texture: None,
                entity_sampler: None,
                entity_set: None,
                gui_pipeline_layout,
                gui_pipeline,
                gui_invert_pipeline_layout,
                gui_invert_pipeline,
                gui_item_pipeline_layout,
                gui_item_pipeline,
                gui_texture: None,
                gui_sampler: None,
                gui_set: None,
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

    #[expect(
        clippy::too_many_lines,
        reason = "Vulkan objects created in the order the API requires"
    )]
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
                for (image, memory, view) in scene.weather_textures {
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                if let Some(sampler) = scene.weather_sampler {
                    self.device.destroy_sampler(sampler, None);
                }
                if let Some((image, memory, view)) = scene.crack_texture {
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                if let Some(sampler) = scene.crack_sampler {
                    self.device.destroy_sampler(sampler, None);
                }
                if let Some((image, memory, view)) = scene.entity_texture {
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                if let Some(sampler) = scene.entity_sampler {
                    self.device.destroy_sampler(sampler, None);
                }
                for (pipeline, layout) in [
                    (scene.sky_pipeline, scene.sky_pipeline_layout),
                    (
                        scene.sky_overlay_pipeline,
                        scene.sky_overlay_pipeline_layout,
                    ),
                    (scene.cloud_pipeline, scene.cloud_pipeline_layout),
                    (scene.weather_pipeline, scene.weather_pipeline_layout),
                    (scene.entity_pipeline, scene.entity_pipeline_layout),
                    (scene.lines_pipeline, scene.lines_pipeline_layout),
                    (scene.held_pipeline, scene.held_pipeline_layout),
                    (scene.crumbling_pipeline, scene.crumbling_pipeline_layout),
                    (scene.text_pipeline, scene.text_pipeline_layout),
                    (
                        scene.text_see_through_pipeline,
                        scene.text_see_through_pipeline_layout,
                    ),
                    (
                        scene.text_translucent_pipeline,
                        scene.text_translucent_pipeline_layout,
                    ),
                    (scene.shadow_pipeline, scene.shadow_pipeline_layout),
                    (scene.swirl_pipeline, scene.swirl_pipeline_layout),
                    (scene.dissolve_pipeline, scene.dissolve_pipeline_layout),
                    (scene.rays_pipeline, scene.rays_pipeline_layout),
                    (scene.gui_pipeline, scene.gui_pipeline_layout),
                    (scene.gui_invert_pipeline, scene.gui_invert_pipeline_layout),
                    (scene.gui_item_pipeline, scene.gui_item_pipeline_layout),
                ] {
                    self.device.destroy_pipeline(pipeline, None);
                    self.device.destroy_pipeline_layout(layout, None);
                }
                if let Some((image, memory, view)) = scene.gui_texture {
                    self.device.destroy_image_view(view, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                if let Some(sampler) = scene.gui_sampler {
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
    /// Hand over the ten crack stages, stacked top to bottom in one image.
    pub fn load_crack_texture(&mut self, stages: &DecodedImage) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.crack_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, sampler, set)) = self.upload_sampled_texture(stages)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.crack_texture = Some(texture);
        scene.crack_sampler = Some(sampler);
        scene.crack_set = Some(set);
        Ok(())
    }

    /// Hand over the next frame's breaking overlay, placed like the
    /// terrain's.
    pub fn set_breaking(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.breaking_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.breaking_buffer = buffer;
        self.breaking_count = vertices.len() as u32;
        Ok(())
    }

    pub fn load_entity_texture(&mut self, skin: &DecodedImage) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.entity_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, sampler, set)) = self.upload_sampled_texture(skin)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.entity_texture = Some(texture);
        scene.entity_sampler = Some(sampler);
        scene.entity_set = Some(set);
        Ok(())
    }

    /// Load the GUI sprites' atlas, sampled texel by texel.
    pub fn load_gui_texture(&mut self, atlas: &DecodedImage) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.gui_set.is_some())
        {
            return Ok(());
        }
        let Some((texture, sampler, set)) = self.upload_sampled_texture(atlas)? else {
            return Ok(());
        };
        let scene = self.scene.as_mut().expect("checked above");
        scene.gui_texture = Some(texture);
        scene.gui_sampler = Some(sampler);
        scene.gui_set = Some(set);
        Ok(())
    }

    /// Upload one texture with a nearest, clamped sampler and a descriptor
    /// set for it.
    #[expect(
        clippy::type_complexity,
        reason = "the handles one upload creates, returned together"
    )]
    fn upload_sampled_texture(
        &mut self,
        skin: &DecodedImage,
    ) -> Result<
        Option<(
            (vk::Image, vk::DeviceMemory, vk::ImageView),
            vk::Sampler,
            vk::DescriptorSet,
        )>,
        VulkanError,
    > {
        unsafe {
            let Some(layout) = self.scene.as_ref().map(|scene| scene.descriptor_layout) else {
                return Ok(None);
            };
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
            Ok(Some(((texture, memory, view), sampler, set)))
        }
    }

    /// Hand over the next frame's entity triangles, placed like the
    /// terrain's, with each vertex's normal in its tint.
    pub fn set_entities(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.entity_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.entity_buffer = buffer;
        self.entity_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's glowing layers, as triangles that sample the
    /// entity textures, each vertex's tint its colour.
    pub fn set_entity_glow(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.glow_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.glow_buffer = buffer;
        self.glow_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's energy swirls: each vertex carries its
    /// texture's place, the offset and the colour (see [`SWIRL_VERTEX_SHADER`]).
    pub fn set_entity_swirl(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.swirl_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.swirl_buffer = buffer;
        self.swirl_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's dissolving entities (a dying dragon's body): the
    /// entity textures' triangles, each vertex's shade its alpha, and the offset
    /// from a texel of the texture to the same texel of its mask in the packed image.
    pub fn set_entity_dissolve(
        &mut self,
        vertices: &[Vertex],
        mask_offset: [f32; 2],
    ) -> Result<(), VulkanError> {
        let mut buffer = self.dissolve_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.dissolve_buffer = buffer;
        self.dissolve_count = vertices.len() as u32;
        self.dissolve_mask_offset = mask_offset;
        Ok(())
    }

    /// Hand over the next frame's dragon rays: each vertex's tint its colour.
    pub fn set_dragon_rays(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.rays_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.rays_buffer = buffer;
        self.rays_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's dropped items: triangles placed like the
    /// terrain's, each with its shade and colour, sampling the block atlas.
    pub fn set_ground_items(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.ground_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.ground_buffer = buffer;
        self.ground_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's entity shadows, as triangles that sample the
    /// entity textures.
    pub fn set_shadows(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.shadow_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.shadow_buffer = buffer;
        self.shadow_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's experience orbs, as triangles that sample the
    /// entity textures.
    pub fn set_experience_orbs(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = self.orb_buffer.take();
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.orb_buffer = buffer;
        self.orb_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's name tags: the text seen through blocks,
    /// which is drawn first, and the text depth-tested. Both sample the GUI
    /// atlas the font's images are in.
    pub fn set_name_tags(
        &mut self,
        see_through: &[Vertex],
        normal: &[Vertex],
    ) -> Result<(), VulkanError> {
        let mut buffer = self.name_tag_see_buffer.take();
        self.fill_vertex_buffer(&mut buffer, see_through)?;
        self.name_tag_see_buffer = buffer;
        self.name_tag_see_count = see_through.len() as u32;
        let mut buffer = self.name_tag_buffer.take();
        self.fill_vertex_buffer(&mut buffer, normal)?;
        self.name_tag_buffer = buffer;
        self.name_tag_count = normal.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's GUI: triangles in GUI units, their runs by
    /// blend, and the window's size in GUI units.
    pub fn set_gui(
        &mut self,
        frame: &crate::gui::GuiFrame,
        size: [f32; 2],
    ) -> Result<(), VulkanError> {
        let mut buffer = self.gui_buffer.take();
        let bytes = unsafe {
            std::slice::from_raw_parts(
                frame.vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(frame.vertices.as_slice()),
            )
        };
        self.fill_bytes(&mut buffer, bytes)?;
        self.gui_buffer = buffer;
        if !frame.items.is_empty() {
            let mut items = self.gui_item_buffer.take();
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    frame.items.as_ptr().cast::<u8>(),
                    std::mem::size_of_val(frame.items.as_slice()),
                )
            };
            self.fill_bytes(&mut items, bytes)?;
            self.gui_item_buffer = items;
        }
        self.gui_batches = frame.batches.clone();
        self.gui_size = size;
        Ok(())
    }

    /// Hand over the next frame's block outline strips, placed like the
    /// terrain's.
    pub fn set_outline(
        &mut self,
        vertices: &[crate::outline::LineVertex],
    ) -> Result<(), VulkanError> {
        let mut buffer = self.outline_buffer.take();
        let bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(vertices),
            )
        };
        self.fill_bytes(&mut buffer, bytes)?;
        self.outline_buffer = buffer;
        self.outline_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's first-person hand: triangles in view
    /// space, and the projection they are drawn with.
    pub fn set_hand(
        &mut self,
        vertices: &[Vertex],
        held: &[Vertex],
        projection: [[f32; 4]; 4],
    ) -> Result<(), VulkanError> {
        let mut buffer = self.hand_buffer.take();
        let all: Vec<Vertex> = vertices.iter().chain(held).copied().collect();
        self.fill_vertex_buffer(&mut buffer, &all)?;
        self.hand_buffer = buffer;
        self.hand_count = vertices.len() as u32;
        self.held_count = held.len() as u32;
        self.hand_projection = projection;
        Ok(())
    }

    /// Write vertices into a host buffer, growing it when they do not fit.
    fn fill_vertex_buffer(
        &mut self,
        slot: &mut Option<SkyBuffer>,
        vertices: &[Vertex],
    ) -> Result<(), VulkanError> {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(vertices),
            )
        };
        self.fill_bytes(slot, bytes)
    }

    /// Write bytes into a host vertex buffer, growing it when they do not
    /// fit.
    fn fill_bytes(
        &mut self,
        slot: &mut Option<SkyBuffer>,
        bytes: &[u8],
    ) -> Result<(), VulkanError> {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let size = bytes.len();
            if slot.as_ref().is_none_or(|buffer| buffer.capacity < size) {
                if let Some(old) = slot.take() {
                    self.device.destroy_buffer(old.buffer, None);
                    self.device.free_memory(old.memory, None);
                }
                let capacity = size.max(1 << 16);
                let (buffer, memory) =
                    self.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vec![0_u8; capacity])?;
                *slot = Some(SkyBuffer {
                    buffer,
                    memory,
                    capacity,
                });
            }
            let buffer = slot.as_ref().expect("allocated above");
            if size > 0 {
                let mapped = self
                    .device
                    .map_memory(buffer.memory, 0, size as u64, vk::MemoryMapFlags::empty())
                    .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                    .cast::<u8>();
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped, size);
                self.device.unmap_memory(buffer.memory);
            }
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
        vertices: &[crate::clouds::CloudVertex],
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
        vertices: &[crate::sky::SkyVertex],
        batches: &[(crate::sky::SkyBlend, u32, u32)],
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

    #[expect(
        clippy::too_many_lines,
        reason = "one frame submitted in the order the API requires"
    )]
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
            self.trim_staging();
            if capture {
                self.make_readback()?;
            }

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
                    self.readback.expect("made for the capture").0,
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
            let memory = self.readback.expect("made for the capture").1;
            let mapped = self
                .device
                .map_memory(memory, 0, byte_count as u64, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            let mut rgba = vec![0_u8; byte_count];
            std::ptr::copy_nonoverlapping(mapped, rgba.as_mut_ptr(), byte_count);
            self.device.unmap_memory(memory);
            self.free_readback();
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
            for buffer in [
                self.entity_buffer.take(),
                self.ground_buffer.take(),
                self.name_tag_see_buffer.take(),
                self.orb_buffer.take(),
                self.shadow_buffer.take(),
                self.glow_buffer.take(),
                self.swirl_buffer.take(),
                self.dissolve_buffer.take(),
                self.rays_buffer.take(),
                self.name_tag_buffer.take(),
                self.hand_buffer.take(),
                self.outline_buffer.take(),
                self.breaking_buffer.take(),
                self.gui_buffer.take(),
                self.gui_item_buffer.take(),
            ]
            .into_iter()
            .flatten()
            {
                self.device.destroy_buffer(buffer.buffer, None);
                self.device.free_memory(buffer.memory, None);
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
            self.free_readback();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The shaders are compiled when a pipeline is made, which needs a device;
    /// the entity pair is compiled here so a change to it fails the gate.
    #[test]
    fn the_entity_shaders_compile() {
        let compiler = shaderc::Compiler::new().expect("a shader compiler");
        for (source, kind) in [
            (ENTITY_VERTEX_SHADER, shaderc::ShaderKind::Vertex),
            (ENTITY_FRAGMENT_SHADER, shaderc::ShaderKind::Fragment),
            (DISSOLVE_FRAGMENT_SHADER, shaderc::ShaderKind::Fragment),
            (RAYS_VERTEX_SHADER, shaderc::ShaderKind::Vertex),
            (RAYS_FRAGMENT_SHADER, shaderc::ShaderKind::Fragment),
            (SECTION_VERTEX_SHADER, shaderc::ShaderKind::Vertex),
            (HELD_VERTEX_SHADER, shaderc::ShaderKind::Vertex),
            (FRAGMENT_SHADER, shaderc::ShaderKind::Fragment),
        ] {
            compiler
                .compile_into_spirv(source, kind, "entity", "main", None)
                .unwrap_or_else(|error| panic!("{error}"));
        }
    }
}
