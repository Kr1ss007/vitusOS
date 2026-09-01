// shaders/rounded_rect.frag
// Top-edge highlight: 1px at 8% white — all surfaces altitude Low+
#version 450
layout(push_constant) uniform PC {
    vec2  pos; vec2 size; vec2 screenSize;
    vec4  fillColor; vec4 borderColor;
    float borderWidth; float cornerRadius; float opacity; float _pad;
} pc;
layout(location=0) in vec2 fLocal;
layout(location=1) in vec2 fSize;
layout(location=0) out vec4 outColor;

float sdRR(vec2 p, vec2 hs, float r) {
    vec2 d = abs(p - hs) - hs + vec2(r);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

void main() {
    float d     = sdRR(fLocal, fSize * 0.5, pc.cornerRadius);
    float outer = 1.0 - smoothstep(-0.5, 0.5, d);
    if (outer < 0.001) discard;

    vec4 color;
    if (pc.borderWidth > 0.0) {
        float id    = d + pc.borderWidth;
        float inner = 1.0 - smoothstep(-0.5, 0.5, id);
        color = mix(pc.borderColor, pc.fillColor, inner);
    } else {
        color = pc.fillColor;
    }

    // Top edge highlight — frosted glass surface catches light
    float topHi = (1.0 - smoothstep(0.0, 1.5, fLocal.y)) * 0.08;
    color.rgb += topHi;

    outColor = vec4(color.rgb, color.a * outer * pc.opacity);
}
