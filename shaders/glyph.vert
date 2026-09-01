// shaders/glyph.vert
// Per-glyph quad. HarfBuzz fractional positions PRESERVED — never round.
#version 450
layout(push_constant) uniform PC {
    vec2 screenSize;
    vec2 glyphPos;     // fractional pixel position from HarfBuzz (26.6 / 64.0)
    vec2 glyphSize;    // glyph bitmap dimensions in pixels
    vec2 atlasUVMin;
    vec2 atlasUVMax;
    vec4 textColor;    // premultiplied alpha
} pc;
const vec2 VERTS[4] = vec2[](vec2(0,0), vec2(1,0), vec2(0,1), vec2(1,1));
layout(location=0) out vec2 fAtlasUV;
void main() {
    vec2 lp  = VERTS[gl_VertexIndex];
    vec2 pix = pc.glyphPos + lp * pc.glyphSize;
    vec2 ndc = (pix / pc.screenSize) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    fAtlasUV = mix(pc.atlasUVMin, pc.atlasUVMax, lp);
}
