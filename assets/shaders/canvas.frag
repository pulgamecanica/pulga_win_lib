#version 410 core

in vec2 vUV;
in vec4 vColor;

out vec4 outColor;

uniform sampler2D tex; // RGBA; plain rects sample a 1x1 white texture

void main() {
    outColor = vColor * texture(tex, vUV);
    if (outColor.a <= 0.0) discard;
}
