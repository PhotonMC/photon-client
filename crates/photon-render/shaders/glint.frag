#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) flat in vec4 in_rect;
layout(location = 2) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
    // `glintStrength`, after the boss's two values at 176, and 1 when the version played draws the
    // glint as part of its surface's own draw.
    layout(offset = 184) float glint_strength;
    float in_base_draw;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
#include "glint_sample.glsl"
void main() {
    vec4 colour = glint_sample(skin, in_uv, in_rect);
    if (colour.a < 0.1) { discard; }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    // The blend adds the square of this colour. A pass of its own fades the colour, so the added
    // glint is faded twice. A glint that is part of the surface's draw is added before the
    // surface's fog (item.fsh, entity.fsh), so it is faded once, by the fog colour's share.
    float fade = (1.0 - fog) * (1.0 - fog);
    if (push.in_base_draw > 0.5) {
        fade = 1.0 - fog * push.fog_colour.a;
    }
    out_colour = vec4(colour.rgb * (push.glint_strength * sqrt(fade)), colour.a);
}
