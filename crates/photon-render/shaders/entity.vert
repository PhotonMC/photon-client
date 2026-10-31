#version 450
#include "push.glsl"
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in float in_shade;
layout(location = 3) in vec4 in_tint;
layout(location = 4) in vec2 in_light;

#include "entity_vertex.glsl"
void main() {
    entity_vertex(in_position, in_uv, in_shade, in_tint, in_light);
}
