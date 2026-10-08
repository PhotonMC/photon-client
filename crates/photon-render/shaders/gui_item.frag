#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 0) out vec4 out_colour;
void main() {
    ivec2 size = textureSize(atlas, 0);
    vec4 texel = texelFetch(atlas, clamp(ivec2(floor(in_uv * vec2(size))), ivec2(0), size - 1), 0);
    if (texel.a < 0.1) { discard; }
    out_colour = texel * in_colour;
}
