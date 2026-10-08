#version 450
layout(push_constant) uniform Push {
    mat4 view_projection;
    // The framebuffer's size in pixels.
    vec4 screen;
    vec4 unused_block;
    vec4 unused_ambient;
    vec4 camera;
    vec4 fog_colour;
    vec4 fog_distances;
} push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec3 in_normal;
layout(location = 3) in float in_width;
layout(location = 0) out vec4 out_colour;
layout(location = 1) out vec2 out_fog;
void main() {
    vec4 start = push.view_projection * vec4(in_position, 1.0);
    vec4 end = push.view_projection * vec4(in_position + in_normal, 1.0);
    vec3 ndc_start = start.xyz / start.w;
    vec3 ndc_end = end.xyz / end.w;
    vec2 screen = push.screen.xy;
    vec2 direction = normalize((ndc_end.xy - ndc_start.xy) * screen);
    vec2 offset = vec2(-direction.y, direction.x) * abs(in_width) / screen;
    if (offset.x < 0.0) {
        offset = -offset;
    }
    offset *= sign(in_width);
    gl_Position = vec4((ndc_start + vec3(offset, 0.0)) * start.w, start.w);
    out_colour = in_colour;
    vec3 from_camera = in_position - push.camera.xyz;
    out_fog = vec2(length(from_camera), max(length(from_camera.xz), abs(from_camera.y)));
}
