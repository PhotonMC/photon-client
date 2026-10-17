#version 450
// The end sky and the portal's own picture, each in a set of its own, repeating.
layout(set = 0, binding = 0) uniform sampler2D sky;
layout(set = 1, binding = 0) uniform sampler2D swirl;
layout(location = 0) in vec3 in_screen;
layout(location = 1) in vec2 in_fog;
layout(location = 2) flat in float in_layers;
layout(location = 0) out vec4 out_colour;
layout(push_constant) uniform Push {
    layout(offset = 128) vec4 fog_colour;
    vec4 fog_distances;
    // How far through the day's 24000 ticks the game is, as the reference's global clock.
    float game_time;
} push;

// The tint each layer lays over the picture, darkest first.
const vec3 tints[16] = vec3[16](
    vec3(0.022087, 0.098399, 0.110818),
    vec3(0.011892, 0.095924, 0.089485),
    vec3(0.027636, 0.101689, 0.100326),
    vec3(0.046564, 0.109883, 0.114838),
    vec3(0.064901, 0.117696, 0.097189),
    vec3(0.063761, 0.086895, 0.123646),
    vec3(0.084817, 0.111994, 0.166380),
    vec3(0.097489, 0.154120, 0.091064),
    vec3(0.106152, 0.131144, 0.195191),
    vec3(0.097721, 0.110188, 0.187229),
    vec3(0.133516, 0.138278, 0.148582),
    vec3(0.070006, 0.243332, 0.235792),
    vec3(0.196766, 0.142899, 0.214696),
    vec3(0.047281, 0.315338, 0.321970),
    vec3(0.204675, 0.390010, 0.302066),
    vec3(0.080955, 0.314821, 0.661491));

float linear_fog(float distance, float start, float end) {
    if (distance <= start) { return 0.0; }
    if (distance >= end) { return 1.0; }
    return (distance - start) / (end - start);
}

// Where layer n (from one) looks in the picture for a point of the screen: the point turned and
// scaled by a different amount for each layer, slid along by the clock, then halved and moved to
// the middle of the picture.
vec2 layer_point(vec2 screen, float n) {
    float angle = radians((n * n * 4321.0 + n * 9.0) * 2.0);
    float scale = (4.5 - n / 4.0) * 2.0;
    vec2 turned = scale * vec2(
        cos(angle) * screen.x - sin(angle) * screen.y,
        sin(angle) * screen.x + cos(angle) * screen.y);
    vec2 slid = turned + vec2(17.0 / n, (2.0 + n / 1.5) * (push.game_time * 1.5));
    return slid * 0.5 + 0.25;
}

void main() {
    vec2 screen = in_screen.xy / in_screen.z;
    vec3 colour = texture(sky, screen).rgb * tints[0];
    for (int i = 0; i < 16; i++) {
        if (float(i) >= in_layers) { break; }
        colour += texture(swirl, layer_point(screen, float(i + 1))).rgb * tints[i];
    }
    float fog = max(
        linear_fog(in_fog.x, push.fog_distances.x, push.fog_distances.y),
        linear_fog(in_fog.y, push.fog_distances.z, push.fog_distances.w));
    out_colour = vec4(mix(colour, push.fog_colour.rgb, fog * push.fog_colour.a), 1.0);
}
