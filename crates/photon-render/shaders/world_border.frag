#version 450
layout(set = 0, binding = 0) uniform sampler2D forcefield;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_tint;
layout(location = 0) out vec4 out_colour;
void main() {
    out_colour = texture(forcefield, in_uv) * in_tint;
}
