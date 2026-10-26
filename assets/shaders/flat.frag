#version 410 core

in vec3 vWorldNormal;
in float vViewDistance;

out vec4 outColor;

uniform vec4 color;
uniform vec4 fog; // rgb + density; density 0 = no fog

// One fixed directional light plus ambient, so faces of a flat-colored
// mesh still read as distinct; then exponential-squared distance fog.
void main() {
    vec3 n = normalize(vWorldNormal);
    float ndl = clamp(dot(n, normalize(vec3(0.4, 0.7, 0.6))), 0.0, 1.0);
    float shade = 0.45 + 0.55 * ndl;
    vec3 lit = color.rgb * shade;
    float f = vViewDistance * fog.a;
    outColor = vec4(mix(lit, fog.rgb, 1.0 - exp(-f * f)), color.a);
}
