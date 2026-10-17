// One texel of the reference's 16x16 lightmap, as its lightmap shader
// computes it and as an 8-bit texture stores it.
float brightness(float level) {
    return level / (4.0 - 3.0 * level);
}
vec3 lightmap_texel(int block, int sky) {
    float block_level = float(block) / 15.0;
    float sky_level = float(sky) / 15.0;
    float block_brightness = brightness(block_level) * push.block_tint_factor.a;
    float sky_brightness = brightness(sky_level) * push.sky_colour_factor.a;
    vec3 colour = push.ambient_brightness.rgb;
    colour += push.sky_colour_factor.rgb * sky_brightness;
    float parabola = (2.0 * block_level - 1.0) * (2.0 * block_level - 1.0);
    colour += mix(push.block_tint_factor.rgb, vec3(1.0), 0.9 * parabola) * block_brightness;
    colour -= vec3(push.camera.w);
    colour = clamp(colour, 0.0, 1.0);
    float top = max(max(colour.x, colour.y), colour.z);
    float inverted = 1.0 - top;
    vec3 lifted = colour * ((1.0 - inverted * inverted * inverted * inverted) / top);
    colour = mix(colour, lifted, push.ambient_brightness.a);
    return floor(colour * 255.0 + 0.5) / 255.0;
}
// The lightmap sampled with linear filtering at the vertex's coordinates.
vec3 sample_lightmap(vec2 coords) {
    vec2 uv = clamp(coords / 256.0 + 0.5 / 16.0, vec2(0.5 / 16.0), vec2(15.5 / 16.0));
    vec2 texel = uv * 16.0 - 0.5;
    ivec2 low = ivec2(floor(texel));
    vec2 fraction = texel - floor(texel);
    ivec2 high = min(low + 1, ivec2(15));
    vec3 bottom = mix(lightmap_texel(low.x, low.y), lightmap_texel(high.x, low.y), fraction.x);
    vec3 top = mix(lightmap_texel(low.x, high.y), lightmap_texel(high.x, high.y), fraction.x);
    return mix(bottom, top, fraction.y);
}
