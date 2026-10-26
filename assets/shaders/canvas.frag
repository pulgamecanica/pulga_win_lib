#version 410 core

in vec2 vUV;
in vec4 vColor;
in vec2 vLocal;
flat in vec2 vHalf;
flat in vec3 vShape; // radius, border width, softness
flat in vec4 vBorderColor;

out vec4 outColor;

uniform sampler2D tex; // RGBA; plain rects sample a 1x1 white texture

// Signed distance from p to a box of half size b with corner radius r.
float roundedBox(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    vec4 color = vColor * texture(tex, vUV);
    if (vHalf.x > 0.0) {
        float d = roundedBox(vLocal, vHalf, vShape.x);
        // Half a pixel of anti-aliasing, or a wide falloff for shadows.
        float soft = max(vShape.z, 0.6);
        if (vShape.y > 0.0) {
            color = mix(color, vBorderColor, smoothstep(-soft, soft, d + vShape.y));
        }
        color.a *= 1.0 - smoothstep(-soft, soft, d);
    }
    if (color.a <= 0.0) discard;
    outColor = color;
}
