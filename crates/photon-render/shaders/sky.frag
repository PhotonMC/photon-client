#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(push_constant) uniform Push { layout(offset = 64) vec4 fog_colour; float sky_end; } push;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec2 in_fog;
layout(location = 0) out vec4 out_colour;
void main() {
    vec4 colour = in_colour;
    if (in_uv.x >= 0.0) {
        // Celestial sprites are sampled at the nearest texel of the top level.
        ivec2 size = textureSize(atlas, 0);
        ivec2 at = ivec2(floor(in_uv * vec2(size)));
        // The End sky's cube repeats its texture (marked by -2 in the fog's second distance).
        at = in_fog.y <= -2.0 ? ((at % size) + size) % size : clamp(at, ivec2(0), size - 1);
        vec4 texel = texelFetch(atlas, at, 0);
        if (texel.a == 0.0) { discard; }
        colour *= texel;
    }
    if (in_fog.x >= 0.0) {
        float spherical = in_fog.x <= 0.0 ? 0.0 : (in_fog.x >= push.sky_end ? 1.0 : in_fog.x / push.sky_end);
        float cylindrical = in_fog.y <= push.sky_end ? 0.0 : 1.0;
        float fog = max(spherical, cylindrical);
        colour.rgb = mix(colour.rgb, push.fog_colour.rgb, fog * push.fog_colour.a);
    }
    out_colour = colour;
}
