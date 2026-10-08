#version 450
layout(push_constant) uniform Push {
    layout(offset = 64) vec4 offset_and_end;
} push;
layout(location = 0) in vec4 in_colour;
layout(location = 1) in float in_distance;
layout(location = 0) out vec4 out_colour;
void main() {
    float end = push.offset_and_end.w;
    float fog = in_distance <= 0.0 ? 0.0 : (in_distance >= end ? 1.0 : in_distance / end);
    out_colour = vec4(in_colour.rgb, in_colour.a * (1.0 - fog));
}
