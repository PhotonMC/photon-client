#version 450
#include "push.glsl"
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in float in_shade;
layout(location = 3) in vec4 in_tint;
layout(location = 4) in vec2 in_light;

#include "lightmap.glsl"
layout(location = 0) out vec4 out_colour;
layout(location = 1) out vec2 out_fog;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_colour = in_tint;
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
}
