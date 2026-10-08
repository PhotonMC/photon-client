#version 450
#include "push.glsl"
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_colour;
layout(location = 3) in uvec2 in_light;

#include "lightmap.glsl"
layout(location = 0) out vec2 out_uv;
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
