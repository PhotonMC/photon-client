// The push constants every scene vertex shader reads.
layout(push_constant) uniform Push {
    mat4 view_projection;
    vec4 sky_colour_factor;
    vec4 block_tint_factor;
    vec4 ambient_brightness;
    vec4 camera;
    vec4 fog_colour;
    vec4 fog_distances;
    // The boss bars' darkening of the world (x) and the level's second diffuse light pointing
    // down (y, 1 in the Nether), after the run-specific values at 160.
    layout(offset = 176) vec4 boss;
} push;
