#version 450
#include "push.glsl"
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in float in_shade;
layout(location = 3) in vec4 in_tint;
layout(location = 4) in vec2 in_light;
layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_tint;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    // Unlike the other scene passes, the reference's world border overlay
    // is never lit by the lightmap or thinned by fog: its colour is the
    // status tint and alpha `extract` computed, straight through.
    out_tint = in_tint;
}
