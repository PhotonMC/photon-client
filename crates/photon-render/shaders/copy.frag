#version 450
// The blurred background put on the frame: the texel under each pixel.
layout(set = 0, binding = 0) uniform sampler2D in_sampler;
layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_colour;
void main() {
    out_colour = texture(in_sampler, in_uv);
}
