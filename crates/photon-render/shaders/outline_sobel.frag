#version 450
// The first pass of the entity outline chain: where the silhouettes' alpha changes between a texel
// and the four beside it, an edge, in the average colour of the five.
layout(set = 0, binding = 0) uniform sampler2D in_sampler;
layout(push_constant) uniform Push {
    layout(offset = 0) vec2 in_size;
} push;
layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_colour;
void main() {
    vec2 one_texel = 1.0 / push.in_size;
    vec4 centre = texture(in_sampler, in_uv);
    vec4 left = texture(in_sampler, in_uv - vec2(one_texel.x, 0.0));
    vec4 right = texture(in_sampler, in_uv + vec2(one_texel.x, 0.0));
    vec4 up = texture(in_sampler, in_uv - vec2(0.0, one_texel.y));
    vec4 down = texture(in_sampler, in_uv + vec2(0.0, one_texel.y));
    float total = clamp(
        abs(centre.a - left.a) + abs(centre.a - right.a) + abs(centre.a - up.a) + abs(centre.a - down.a),
        0.0, 1.0);
    vec3 colour = centre.rgb * centre.a + left.rgb * left.a + right.rgb * right.a
        + up.rgb * up.a + down.rgb * down.a;
    out_colour = vec4(colour * 0.2, total);
}
