#version 450
// The box blur of the entity outline chain, radius two, along one direction. The sampler filters
// linearly, so a sample covers two texels; the last one counts half. The colour is averaged over
// the weights and the alpha is their sum, as the reference does it.
layout(set = 0, binding = 0) uniform sampler2D in_sampler;
layout(push_constant) uniform Push {
    layout(offset = 0) vec2 direction;
    layout(offset = 8) vec2 in_size;
} push;
layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_colour;
void main() {
    vec2 step_size = (1.0 / push.in_size) * push.direction;
    const float radius = 2.0;
    vec4 blurred = vec4(0.0);
    for (float a = -radius + 0.5; a <= radius; a += 2.0) {
        blurred += texture(in_sampler, in_uv + step_size * a);
    }
    blurred += texture(in_sampler, in_uv + step_size * radius) / 2.0;
    out_colour = vec4((blurred / (radius + 0.5)).rgb, blurred.a);
}
