#version 450
layout(set = 0, binding = 0) uniform samplerCube cube;
layout(location = 0) in vec3 in_direction;
layout(location = 0) out vec4 out_colour;
void main() {
    out_colour = texture(cube, in_direction);
}
