// shaders/window_shadow.frag
// SDF dual shadow. Warm shadow color #1A1208. Never #000000.
// shadowPos is SPRING_SHADOW (300,25) lagged — creates depth perception.
// Ambient: large spread (glass floats). Contact: tight (grounds bottom edge).
#version 450
layout(push_constant) uniform PC {
    vec2  screenSize;
    vec2  shadowPos;     // SPRING_SHADOW spring-lagged position
    vec2  windowSize;
    float cornerRadius;
    float _pad;
} pc;
layout(location=0) in  vec2 fragCoord;
layout(location=0) out vec4 outColor;

float sdRR(vec2 p, vec2 hs, float r) {
    vec2 d = abs(p - hs) - hs + vec2(r);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

void main() {
    vec2 center = pc.shadowPos + pc.windowSize * 0.5;
    float d = sdRR(fragCoord - center, pc.windowSize * 0.5, pc.cornerRadius);
    if (d < 0.0) discard;  // inside window rect — no shadow there

    float ambient = exp(-d / 40.0) * 0.18;   // spread 60px, soft blur 40px, 18% peak
    float contact = exp(-d /  8.0) * 0.12;   // tight 8px, grounds bottom edge 12% peak
    float shadow  = clamp(ambient + contact, 0.0, 1.0);

    // #1A1208 = rgb(0.102, 0.071, 0.031) — warm dark, never pure black
    outColor = vec4(0.102, 0.071, 0.031, shadow);
}
