// The push constants every scene vertex shader reads.
layout(push_constant) uniform Push {
    mat4 view_projection;
    vec4 sky_colour_factor;
    vec4 block_tint_factor;
    vec4 ambient_brightness;
    vec4 camera;
    vec4 fog_colour;
    vec4 fog_distances;
    // The boss bars' darkening of the world (x), after the run-specific values at 160.
    layout(offset = 176) vec4 boss;
} push;
