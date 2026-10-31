#version 450
#include "push.glsl"
// The model-space mesh, shared by every instance of it.
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 3) in vec4 in_tint;
// One instance: the placement (the columns of the position matrix, translation last), the
// normal matrix, then the light, the shade (overlay code) and the colour.
layout(location = 5) in vec3 in_position_x;
layout(location = 6) in vec3 in_position_y;
layout(location = 7) in vec3 in_position_z;
layout(location = 8) in vec3 in_position_w;
layout(location = 9) in vec3 in_normal_x;
layout(location = 10) in vec3 in_normal_y;
layout(location = 11) in vec3 in_normal_z;
layout(location = 12) in vec2 in_shade_colour;
layout(location = 13) in vec2 in_light;

#include "entity_vertex.glsl"
void main() {
    vec3 position = mat3(in_position_x, in_position_y, in_position_z) * in_position + in_position_w;
    vec3 normal = mat3(in_normal_x, in_normal_y, in_normal_z) * in_tint.xyz;
    entity_vertex(position, in_uv, in_shade_colour.x, vec4(normal, in_shade_colour.y), in_light);
}
