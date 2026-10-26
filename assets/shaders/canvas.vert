#version 410 core

// Pixels, origin at the window's top-left corner, y pointing down.
layout(location = 0) in vec2 inPos;
layout(location = 1) in vec2 inUV;
layout(location = 2) in vec4 inColor;
// Rounded-rect shape (all zero for plain textured quads):
layout(location = 3) in vec2 inLocal;      // this vertex relative to the shape's center
layout(location = 4) in vec2 inHalf;       // shape half size; x <= 0 means "no shape"
layout(location = 5) in vec3 inShape;      // corner radius, border width, edge softness
layout(location = 6) in vec4 inBorderColor;

out vec2 vUV;
out vec4 vColor;
out vec2 vLocal;
flat out vec2 vHalf;
flat out vec3 vShape;
flat out vec4 vBorderColor;

uniform vec2 screen; // window size in pixels

void main() {
    vUV = inUV;
    vColor = inColor;
    vLocal = inLocal;
    vHalf = inHalf;
    vShape = inShape;
    vBorderColor = inBorderColor;
    vec2 ndc = vec2(inPos.x / screen.x * 2.0 - 1.0, 1.0 - inPos.y / screen.y * 2.0);
    gl_Position = vec4(ndc, 0.0, 1.0);
}
