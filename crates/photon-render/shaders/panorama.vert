#version 450
// The title screen's cube (`CubeMap`): six faces of corners in the reference's order, each as two
// triangles; the corner doubles as the direction the cube map is sampled in.
layout(push_constant) uniform Push { mat4 view_projection; } push;
layout(location = 0) out vec3 out_direction;
const vec3 CORNERS[24] = vec3[24](
    vec3(-1.0, -1.0, 1.0), vec3(-1.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(1.0, -1.0, 1.0),
    vec3(1.0, -1.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, -1.0), vec3(1.0, -1.0, -1.0),
    vec3(1.0, -1.0, -1.0), vec3(1.0, 1.0, -1.0), vec3(-1.0, 1.0, -1.0), vec3(-1.0, -1.0, -1.0),
    vec3(-1.0, -1.0, -1.0), vec3(-1.0, 1.0, -1.0), vec3(-1.0, 1.0, 1.0), vec3(-1.0, -1.0, 1.0),
    vec3(-1.0, -1.0, -1.0), vec3(-1.0, -1.0, 1.0), vec3(1.0, -1.0, 1.0), vec3(1.0, -1.0, -1.0),
    vec3(-1.0, 1.0, 1.0), vec3(-1.0, 1.0, -1.0), vec3(1.0, 1.0, -1.0), vec3(1.0, 1.0, 1.0));
const int QUAD_ORDER[6] = int[6](0, 1, 2, 0, 2, 3);
void main() {
    vec3 corner = CORNERS[gl_VertexIndex / 6 * 4 + QUAD_ORDER[gl_VertexIndex % 6]];
    gl_Position = push.view_projection * vec4(corner, 1.0);
    out_direction = corner;
}
