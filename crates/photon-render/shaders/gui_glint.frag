#version 450
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 1) flat in vec4 in_rect;
layout(location = 0) out vec4 out_colour;
// The GUI's scale takes `glintStrength` as its third value.
layout(push_constant) uniform Push {
    vec4 scale;
} push;
#include "glint_sample.glsl"
void main() {
    vec4 colour = glint_sample(skin, in_uv, in_rect);
    if (colour.a < 0.1) { discard; }
    out_colour = vec4(colour.rgb * push.scale.z, colour.a);
}
