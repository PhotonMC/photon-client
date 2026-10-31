#version 450
#include "push.glsl"
// The position (xyz, in 1/2048 block from the section's corner) and the light (w, block in the
// low byte, sky in the high one).
layout(location = 0) in uvec4 in_packed;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_colour;

#include "lightmap.glsl"
layout(location = 0) out vec2 out_uv;
layout(location = 1) out float out_shade;
layout(location = 2) out vec4 out_tint;
layout(location = 3) out vec2 out_fog;
layout(location = 4) out vec3 out_light;
void main() {
    ivec3 corner = ivec3(
        int(push.section_origin.x << 16) >> 16,
        int(push.section_origin.x) >> 16,
        int(push.section_origin.y << 16) >> 16);
    vec3 in_position = vec3(corner) + vec3(in_packed.xyz) * (1.0 / 2048.0);
    uvec2 in_light = uvec2(in_packed.w & 255u, in_packed.w >> 8);
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
