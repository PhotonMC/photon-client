#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in float in_shade;
layout(location = 2) in vec4 in_tint;
layout(location = 3) in vec2 in_fog;
layout(location = 4) in vec3 in_light;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
    // How far the section has faded in from the fog colour.
    float visibility;
    // 1 when the terrain is filtered with the rotated grid; 0 for every other run.
    float rgss;
} push;
float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}
// The terrain sampler filters linearly, yet the reference draws blocks
// pixel-sharp: its terrain shader pulls each lookup towards the centre of the
// texel it lands in, by as much as that texel is larger than a screen pixel.
// Up close the lookup sits on the texel centre and reads it alone; only in the
// last screen pixel of a texel edge does it blend with the neighbour, which is
// what keeps the edges from shimmering. Far away, where a texel is smaller
// than a pixel, the lookup is left alone and the mip chain does the work.
vec4 sample_texel_centred(vec2 uv) {
    vec2 texel_size = 1.0 / vec2(textureSize(atlas, 0));
    vec2 du = dFdx(uv);
    vec2 dv = dFdy(uv);
    vec2 texel_on_screen = sqrt(du * du + dv * dv);
    vec2 texel_coords = uv / texel_size;
    vec2 corner = round(texel_coords) - 0.5;
    vec2 offset = texel_coords - corner;
    offset = clamp((offset - 0.5) * texel_size / texel_on_screen + 0.5, 0.0, 1.0);
    return textureGrad(atlas, (corner + offset) * texel_size, du, dv);
}
// `TextureFilteringMethod.RGSS`: where a texel covers between one and two screen pixels, the
// texel-centred lookup gives way to the mean of four lookups spread over a rotated grid a texel
// wide, each read at the mip level the pixel's footprint asks for and blended with the next one.
vec4 sample_rotated_grid(vec2 uv) {
    vec2 texel_size = 1.0 / vec2(textureSize(atlas, 0));
    vec2 du = dFdx(uv);
    vec2 dv = dFdy(uv);
    vec2 footprint = sqrt(du * du + dv * dv);
    float texel = min(texel_size.x, texel_size.y);
    float far = smoothstep(texel, 2.0 * texel, max(footprint.x, footprint.y));
    float level = max(0.0, log2(sqrt(length(du) * length(dv)) / texel));
    float lower = floor(level);
    const vec2 grid[4] = vec2[](
        vec2(0.125, 0.375), vec2(-0.125, -0.375), vec2(0.375, -0.125), vec2(-0.375, 0.125));
    vec4 at_lower = vec4(0.0);
    vec4 at_upper = vec4(0.0);
    for (int i = 0; i < 4; ++i) {
        vec2 at = uv + grid[i] * texel_size;
        at_lower += textureLod(atlas, at, lower);
        at_upper += textureLod(atlas, at, lower + 1.0);
    }
    vec4 spread = mix(at_lower, at_upper, fract(level)) * 0.25;
    return mix(sample_texel_centred(uv), spread, far);
}
void main() {
    vec4 sampled = push.rgss > 0.5 ? sample_rotated_grid(in_uv) : sample_texel_centred(in_uv);
    vec4 texel = sampled * in_tint;
    // The shade carries the entity overlay (`OverlayTexture`) above two: the
    // shade itself is below two, so the whole part of half of it is the
    // overlay's code, as the entities' shader reads it.
    float overlay = floor(in_shade / 2.0);
    texel.rgb *= in_shade - overlay * 2.0;
    if (mod(overlay, 2.0) > 0.5) {
        texel.rgb = mix(vec3(1.0, 0.0, 0.0), texel.rgb, 178.0 / 255.0);
    } else if (overlay > 1.5) {
        float flash = floor((1.0 - floor(overlay / 2.0) / 15.0 * 0.75) * 255.0) / 255.0;
        texel.rgb = mix(vec3(1.0), texel.rgb, flash);
    }
    texel.rgb *= in_light;
    texel = mix(vec4(push.fog_colour.rgb, texel.a), texel, push.visibility);
    if (texel.a < 0.1) { discard; }
    vec3 colour = texel.rgb;
    // The environment's fog over the spherical distance, the render
    // distance's over the cylindrical one, whichever is thicker.
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour, push.fog_colour.rgb, fog * push.fog_colour.a), texel.a);
}
