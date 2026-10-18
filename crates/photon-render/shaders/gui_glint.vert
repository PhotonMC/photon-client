#version 450
layout(push_constant) uniform Push {
    vec4 scale;
} push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_rect;
layout(location = 0) out vec2 out_uv;
layout(location = 1) flat out vec4 out_rect;
invariant gl_Position;
void main() {
    // The item's own vertex mapping, so the glint's depth equals what the item wrote.
    gl_Position = vec4(in_position.xy * push.scale.xy - 1.0, 0.5 + in_position.z / 2000.0, 1.0);
    out_uv = in_uv;
    out_rect = in_rect;
}
