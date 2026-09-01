// shaders/kawase_blur.frag
// One Kawase pass. Run 4× with iterations 0.5, 1.5, 2.5, 3.5.
// 4 passes = ~16 samples. Approximates Gaussian σ≈8px.
// For altitude High/Floating: run two full chains (8 passes total).
// Process in linear RGB — gamma-correct blur. Output back to sRGB.
#version 450
layout(binding=0) uniform sampler2D src;
layout(push_constant) uniform PC {
    vec2  texelSize;   // 1.0 / vec2(outputWidth, outputHeight)
    float iter;        // 0.5, 1.5, 2.5, or 3.5
    float _pad;
} pc;
layout(location=0) in  vec2 fUV;
layout(location=0) out vec4 outColor;

vec3 toLinear(vec3 c) { return c * c; }          // fast γ≈2.0
vec3 toSrgb(vec3 c)   { return sqrt(max(c, 0.0)); }

void main() {
    vec2 off = pc.texelSize * pc.iter;
    vec4 s = (texture(src, fUV + vec2( off.x,  off.y))
            + texture(src, fUV + vec2(-off.x,  off.y))
            + texture(src, fUV + vec2( off.x, -off.y))
            + texture(src, fUV + vec2(-off.x, -off.y))) * 0.25;
    outColor = vec4(toSrgb(toLinear(s.rgb)), s.a);
}
