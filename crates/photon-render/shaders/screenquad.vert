#version 450
// `core/screenquad`: one triangle over the whole target, its texture coordinate from the index.
layout(location = 0) out vec2 out_uv;
void main() {
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
    out_uv = uv;
}
