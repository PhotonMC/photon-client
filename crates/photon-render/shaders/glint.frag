#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) flat in vec4 in_rect;
layout(location = 2) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
} push;
const float GLINT_ALPHA = 0.75;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
vec4 glint_texel(vec2 origin, vec2 size, vec2 at) {
    vec2 wrapped = mod(at, size);
    return texelFetch(skin, ivec2(origin + wrapped), 0);
}
void main() {
    // The glint textures say `blur: true`: linear filtering, repeating at the
    // texture's own edge, which the atlas cannot do for a rect inside it.
    vec2 atlas = vec2(textureSize(skin, 0));
    vec2 origin = round(in_rect.xy * atlas);
    vec2 size = round(in_rect.zw * atlas);
    vec2 texel = ((in_uv - in_rect.xy) / in_rect.zw) * size - 0.5;
    vec2 base = floor(texel);
    vec2 weight = texel - base;
    vec4 colour = mix(
        mix(glint_texel(origin, size, base), glint_texel(origin, size, base + vec2(1.0, 0.0)), weight.x),
        mix(glint_texel(origin, size, base + vec2(0.0, 1.0)), glint_texel(origin, size, base + vec2(1.0)), weight.x),
        weight.y);
    if (colour.a < 0.1) { discard; }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(colour.rgb * ((1.0 - fog) * GLINT_ALPHA), colour.a);
}
