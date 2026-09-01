// shaders/texture_quad.frag
#version 450
layout(binding=0) uniform sampler2D tex;
layout(push_constant) uniform PC {
    vec2 pos; vec2 size; vec2 screenSize;
    float opacity; float cornerRadius;
} pc;
layout(location=0) in vec2 fUV;
layout(location=1) in vec2 fLocal;
layout(location=2) in vec2 fSize;
layout(location=0) out vec4 outColor;

float sdRR(vec2 p, vec2 hs, float r) {
    vec2 d = abs(p - hs) - hs + vec2(r);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

void main() {
    vec4 c = texture(tex, fUV);
    if (pc.cornerRadius > 0.0) {
        float d = sdRR(fLocal, fSize * 0.5, pc.cornerRadius);
        c.a *= 1.0 - smoothstep(-0.5, 0.5, d);
    }
    outColor = vec4(c.rgb, c.a * pc.opacity);
}
