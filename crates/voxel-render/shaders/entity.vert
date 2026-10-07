#version 450
#include "push.glsl"
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in float in_shade;
layout(location = 3) in vec4 in_tint;
layout(location = 4) in vec2 in_light;

#include "lightmap.glsl"
layout(location = 0) out vec2 out_uv;
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
