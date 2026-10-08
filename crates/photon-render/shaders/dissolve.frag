#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec3 in_front;
layout(location = 2) in vec3 in_back;
layout(location = 3) in vec3 in_light;
layout(location = 4) in vec2 in_fog;
layout(location = 5) in float in_alpha_mask;
layout(location = 6) in vec3 in_colour;
layout(location = 7) in float in_unused;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
    vec4 dissolve;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
void main() {
    vec4 colour = texture(skin, in_uv);
    if (colour.a < 0.1) { discard; }
    if (in_alpha_mask < texture(skin, in_uv + push.dissolve.xy).a) { discard; }
    colour.rgb *= in_colour * (gl_FrontFacing ? in_front : in_back);
    colour.rgb *= in_light;
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
