//! The GLSL the pipelines are built from, compiled at start-up.

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
pub(super) const SECTION_VERTEX_SHADER: &str = concat!(
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

pub(super) const VERTEX_SHADER: &str = concat!(
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
pub(super) const HELD_VERTEX_SHADER: &str = concat!(
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
pub(super) const ENTITY_VERTEX_SHADER: &str = concat!(
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
invariant gl_Position;
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
pub(super) const ENTITY_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const TEXT_VERTEX_SHADER: &str = concat!(
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
pub(super) const SWIRL_VERTEX_SHADER: &str = concat!(
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
pub(super) const SWIRL_FRAGMENT_SHADER: &str = r"#version 450
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

/// The glint's vertex shader (`RenderPipelines.GLINT`): the entity's position (the same
/// expression, so the depth is exactly the model's), the texture coordinate already moved by the
/// glint's texture matrix, and the glint texture's place in the packed image in the tint.
pub(super) const GLINT_VERTEX_SHADER: &str = concat!(
    vertex_prelude!(vertex_inputs!()),
    r"invariant gl_Position;
layout(location = 0) out vec2 out_uv;
layout(location = 1) flat out vec4 out_rect;
layout(location = 2) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_rect = in_tint;
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
",
);

/// The glint's fragment shader (`core/glint`): the texture wrapped inside its own place in the
/// packed image (the reference's texture repeats), cut out below a tenth of alpha, its colour
/// faded by fog and by the glint strength (the option's default).
pub(super) const GLINT_FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) flat in vec4 in_rect;
layout(location = 2) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
} push;
const float GLINT_ALPHA = 0.75;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
void main() {
    vec2 local = (in_uv - in_rect.xy) / in_rect.zw;
    vec2 wrapped = in_rect.xy + fract(local) * in_rect.zw;
    vec4 colour = texture(skin, wrapped);
    if (colour.a < 0.1) { discard; }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(colour.rgb * ((1.0 - fog) * GLINT_ALPHA), colour.a);
}
";

/// The dissolving entity's fragment shader (`ENTITY_CUTOUT_DISSOLVE`): the entity's, with
/// no overlay, and each texel dropped where the vertex's alpha (the shade, which
/// carries it) is below the mask's. The mask is another texture of the packed
/// image, at the same place in it moved by the push constant's offset.
pub(super) const DISSOLVE_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const RAYS_VERTEX_SHADER: &str = concat!(
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
pub(super) const RAYS_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const SHADOW_VERTEX_SHADER: &str = concat!(
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
pub(super) const SHADOW_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const TEXT_SEE_THROUGH_VERTEX_SHADER: &str = concat!(
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
pub(super) const TEXT_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const TEXT_SEE_THROUGH_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const LINE_VERTEX_SHADER: &str = r"#version 450
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

pub(super) const LINE_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const CRUMBLING_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const GUI_VERTEX_SHADER: &str = r"#version 450
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

pub(super) const GUI_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const GUI_ITEM_VERTEX_SHADER: &str = r"#version 450
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

pub(super) const GUI_ITEM_FRAGMENT_SHADER: &str = r"#version 450
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

pub(super) const FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const SKY_VERTEX_SHADER: &str = r"#version 450
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

pub(super) const SKY_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const CLOUD_VERTEX_SHADER: &str = r"#version 450
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

pub(super) const CLOUD_FRAGMENT_SHADER: &str = r"#version 450
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
pub(super) const WEATHER_FRAGMENT_SHADER: &str = r"#version 450
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
