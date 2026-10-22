#version 450
// `core/rendertype_outline`: an entity's silhouette in the colour of its outline, cut out by the
// texture's alpha. The vertex shader's colour is the outline colour.
layout(set = 0, binding = 0) uniform sampler2D skin;
layout(location = 0) in vec2 in_uv;
layout(location = 6) in vec3 in_colour;
layout(location = 0) out vec4 out_colour;
void main() {
    if (texture(skin, in_uv).a == 0.0) { discard; }
    out_colour = vec4(in_colour, 1.0);
}
