#version 450
layout(set = 0, binding = 0) uniform sampler2D font;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
void main() {
    vec4 texel = in_uv.x < 0.0 ? vec4(1.0) : texture(font, in_uv);
    vec4 colour = texel * in_colour;
    if (colour.a < 0.1) { discard; }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
