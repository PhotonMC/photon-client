#version 450
// `post/box_blur`: the menu background's blur along one direction. The sampler filters linearly,
// so one sample covers two texels; the last texel is sampled at half weight.
layout(set = 0, binding = 0) uniform sampler2D in_sampler;
layout(push_constant) uniform Push {
    layout(offset = 0) vec2 direction;
    layout(offset = 8) float radius;
    layout(offset = 16) vec2 in_size;
} push;
layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_colour;
void main() {
    vec2 step_size = (1.0 / push.in_size) * push.direction;
    float actual_radius = round(push.radius);
    vec4 blurred = vec4(0.0);
    for (float a = -actual_radius + 0.5; a <= actual_radius; a += 2.0) {
        blurred += texture(in_sampler, in_uv + step_size * a);
    }
    blurred += texture(in_sampler, in_uv + step_size * actual_radius) / 2.0;
    out_colour = blurred / (actual_radius + 0.5);
}
