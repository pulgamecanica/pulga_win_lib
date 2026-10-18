#version 410 core

in vec3 vWorldNormal;
flat in uint vFaceId;

out vec4 outColor;

// Per-face deterministic color (hash + desaturate) so faces are visually
// distinguishable without needing a texture yet.
vec3 faceColor(uint id) {
    uint x = id * 1664525u + 1013904223u;
    float r = float((x >> 0) & 255u) / 255.0;
    float g = float((x >> 8) & 255u) / 255.0;
    float b = float((x >> 16) & 255u) / 255.0;
    vec3 c = vec3(r, g, b);
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    c = mix(vec3(l), c, 0.25);
    return mix(vec3(0.2), c, 0.7);
}

void main() {
    vec3 n = normalize(vWorldNormal);
    float ndl = clamp(dot(n, normalize(vec3(0.4, 0.7, 0.6))), 0.0, 1.0);
    float shade = 0.4 + 0.6 * ndl;
    outColor = vec4(faceColor(vFaceId) * shade, 1.0);
}
