// shaders/texture_quad.vert
// Fullscreen quad from push constants — no VBO.
// Used for: wallpaper, thumbnails, wlr_surface textures, images.
#version 450

layout(push_constant) uniform PC {
    vec2  pos;           // top-left, screen pixels
    vec2  size;          // width/height in pixels
    vec2  screenSize;    // output resolution for NDC
    float opacity;
    float cornerRadius;
} pc;

const vec2 VERTS[4] = vec2[](vec2(0,0), vec2(1,0), vec2(0,1), vec2(1,1));
const vec2 UVS[4]   = vec2[](vec2(0,0), vec2(1,0), vec2(0,1), vec2(1,1));

layout(location=0) out vec2 fUV;
layout(location=1) out vec2 fLocal;
layout(location=2) out vec2 fSize;

void main() {
    vec2 lp  = VERTS[gl_VertexIndex];
    vec2 pix = pc.pos + lp * pc.size;
    vec2 ndc = (pix / pc.screenSize) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    fUV    = UVS[gl_VertexIndex];
    fLocal = lp * pc.size;
    fSize  = pc.size;
}
