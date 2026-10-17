#version 450
#include "push.glsl"
layout(location = 0) in vec3 in_position;
// The first tint component holds how many layers of the surface the block shows.
layout(location = 3) in vec4 in_tint;

// The vertex's place on the screen as texture coordinates, homogeneous so that it interpolates
// perspective-correctly: x and y over the window's width and height, and the clip's w.
layout(location = 0) out vec3 out_screen;
layout(location = 1) out vec2 out_fog;
layout(location = 2) flat out float out_layers;
void main() {
    vec4 clip = push.view_projection * vec4(in_position, 1.0);
    gl_Position = clip;
    // Vulkan's clip space points down where the reference's points up, so y is turned over to put
    // the first row at the bottom of the window as the reference does.
    out_screen = vec3(0.5 * clip.x + 0.5 * clip.w, -0.5 * clip.y + 0.5 * clip.w, clip.w);
    vec3 offset = in_position - push.camera.xyz;
    out_fog = vec2(length(offset), max(length(offset.xz), abs(offset.y)));
    out_layers = in_tint.x;
}
