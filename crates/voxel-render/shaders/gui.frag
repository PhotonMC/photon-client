#version 450
layout(set = 0, binding = 0) uniform sampler2D sprites;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 0) out vec4 out_colour;
void main() {
    vec4 colour = texture(sprites, in_uv) * in_colour;
    if (colour.a == 0.0) { discard; }
    out_colour = colour;
}
