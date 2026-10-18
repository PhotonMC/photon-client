// The glint texture's sample: linear filtering that repeats at the texture's own edge, inside
// the packed image (the glint textures say `blur: true`).
vec4 glint_texel(sampler2D image, vec2 origin, vec2 size, vec2 at) {
    vec2 wrapped = mod(at, size);
    return texelFetch(image, ivec2(origin + wrapped), 0);
}
vec4 glint_sample(sampler2D image, vec2 uv, vec4 rect) {
    vec2 atlas = vec2(textureSize(image, 0));
    vec2 origin = round(rect.xy * atlas);
    vec2 size = round(rect.zw * atlas);
    vec2 texel = ((uv - rect.xy) / rect.zw) * size - 0.5;
    vec2 base = floor(texel);
    vec2 weight = texel - base;
    return mix(
        mix(glint_texel(image, origin, size, base), glint_texel(image, origin, size, base + vec2(1.0, 0.0)), weight.x),
        mix(glint_texel(image, origin, size, base + vec2(0.0, 1.0)), glint_texel(image, origin, size, base + vec2(1.0)), weight.x),
        weight.y);
}
