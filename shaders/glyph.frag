// shaders/glyph.frag
// atlas: VK_FORMAT_R8_UNORM — red channel = FreeType/fontdue coverage.
// Blend state: SRC_ALPHA / ONE_MINUS_SRC_ALPHA
#version 450
layout(binding=0) uniform sampler2D atlas;
layout(push_constant) uniform PC {
    vec2 screenSize; vec2 glyphPos; vec2 glyphSize;
    vec2 atlasUVMin; vec2 atlasUVMax;
    vec4 textColor;
} pc;
layout(location=0) in  vec2 fAtlasUV;
layout(location=0) out vec4 outColor;
void main() {
    float coverage = texture(atlas, fAtlasUV).r;
    outColor = vec4(pc.textColor.rgb, pc.textColor.a * coverage);
}
