// shaders/rounded_rect.vert
// All AnimusEngine AESurfaces (AEWindow chrome, AEContent, pills, buttons, etc.)
#version 450
layout(push_constant) uniform PC {
    vec2  pos; vec2 size; vec2 screenSize;
    vec4  fillColor; vec4 borderColor;
    float borderWidth; float cornerRadius; float opacity; float _pad;
} pc;
const vec2 VERTS[4] = vec2[](vec2(0,0), vec2(1,0), vec2(0,1), vec2(1,1));
layout(location=0) out vec2 fLocal;
layout(location=1) out vec2 fSize;
void main() {
    vec2 lp  = VERTS[gl_VertexIndex];
    vec2 pix = pc.pos + lp * pc.size;
    vec2 ndc = (pix / pc.screenSize) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    fLocal = lp * pc.size;
    fSize  = pc.size;
}
