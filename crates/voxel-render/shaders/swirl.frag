#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) flat in vec4 in_rect;
layout(location = 2) flat in vec2 in_offset;
layout(location = 3) flat in float in_colour;
layout(location = 4) in vec2 in_fog;
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
    vec2 local = (in_uv - in_rect.xy) / in_rect.zw + in_offset;
    vec2 wrapped = in_rect.xy + fract(local) * in_rect.zw;
    vec4 colour = texture(skin, wrapped);
    if (colour.a < 0.1) { discard; }
    colour.rgb *= vec3(
        mod(floor(in_colour / 65536.0), 256.0),
        mod(floor(in_colour / 256.0), 256.0),
        mod(in_colour, 256.0)) / 255.0;
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a), colour.a);
}
