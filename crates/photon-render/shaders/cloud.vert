#version 450
layout(push_constant) uniform Push {
    mat4 view_projection;
    vec4 offset_and_end;
    vec4 colour;
} push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in float in_shade;
layout(location = 0) out vec4 out_colour;
layout(location = 1) out float out_distance;
void main() {
    vec3 position = in_position + push.offset_and_end.xyz;
    gl_Position = push.view_projection * vec4(position, 1.0);
    out_colour = vec4(vec3(in_shade), 1.0) * push.colour;
    out_distance = length(position);
}
