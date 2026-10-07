#version 450
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
