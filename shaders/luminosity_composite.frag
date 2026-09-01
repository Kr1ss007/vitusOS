// shaders/luminosity_composite.frag
// Composites blurred wallpaper → glass surface.
// OKLab for perceptually correct luminosity + chroma ops.
// Source: Björn Ottosson, https://bottosson.github.io/posts/oklab/ (2020)
#version 450
layout(binding=0) uniform sampler2D blurred;
layout(binding=1) uniform sampler2D noiseTex;
layout(push_constant) uniform PC {
    vec4  tintColor;        // from WallpaperTintSampler (sRGB)
    float tintStrength;     // altitude-driven: 0.05–0.35
    float luminosityBoost;  // OKLab L+ : 0.04–0.12
    float chromaReduce;     // OKLab ab× : 0.08–0.20
    float grainStrength;    // noise: 0.015–0.020
    float opacity;          // altitude surface opacity
    float _p0, _p1, _p2;
} pc;
layout(location=0) in  vec2 fUV;
layout(location=0) out vec4 outColor;

vec3 toLinear(vec3 c) { return c * c; }
vec3 toSrgb(vec3 c)   { return sqrt(clamp(c, 0.0, 1.0)); }

vec3 linToOKLab(vec3 c) {
    float l = 0.4122214708*c.r + 0.5363325363*c.g + 0.0514459929*c.b;
    float m = 0.2119034982*c.r + 0.6806995451*c.g + 0.1073969566*c.b;
    float s = 0.0883024619*c.r + 0.2817188376*c.g + 0.6299787005*c.b;
    float l_ = pow(l, 1.0/3.0), m_ = pow(m, 1.0/3.0), s_ = pow(s, 1.0/3.0);
    return vec3(
        0.2104542553*l_ + 0.7936177850*m_ - 0.0040720468*s_,
        1.9779984951*l_ - 2.4285922050*m_ + 0.4505937099*s_,
        0.0259040371*l_ + 0.7827717662*m_ - 0.8086757660*s_);
}

vec3 OKLabToLin(vec3 lab) {
    float l_ = lab.x + 0.3963377774*lab.y + 0.2158037573*lab.z;
    float m_ = lab.x - 0.1055613458*lab.y - 0.0638541728*lab.z;
    float s_ = lab.x - 0.0894841775*lab.y - 1.2914855480*lab.z;
    float l = l_*l_*l_, m = m_*m_*m_, s = s_*s_*s_;
    return vec3(
         4.0767416621*l - 3.3077115913*m + 0.2309699292*s,
        -1.2684380046*l + 2.6097574011*m - 0.3413193965*s,
        -0.0041960863*l - 0.7034186147*m + 1.7076147010*s);
}

void main() {
    vec4 b   = texture(blurred, fUV);
    vec3 lab = linToOKLab(toLinear(b.rgb));

    // Luminosity boost — brightens darks, frosted glass not smeared glass
    lab.x = clamp(lab.x + pc.luminosityBoost, 0.0, 1.0);
    // Chroma reduction — subtle desaturation
    lab.yz *= (1.0 - pc.chromaReduce);

    vec3 result = OKLabToLin(lab);

    // Wallpaper tint in linear RGB
    result = mix(result, toLinear(pc.tintColor.rgb), pc.tintStrength);

    // Noise grain — prevents banding in smooth glass areas
    float noise = texture(noiseTex, fUV * 400.0).r * 2.0 - 1.0;
    result += noise * pc.grainStrength;

    outColor = vec4(toSrgb(result), pc.opacity);
}
