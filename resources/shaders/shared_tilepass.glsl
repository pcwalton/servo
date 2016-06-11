/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//======================================================================================
// Vertex shader attributes and uniforms
//======================================================================================
#ifdef WR_VERTEX_SHADER
    // Attribute inputs
    in vec4 aPositionRect;
#endif

//======================================================================================
// Fragment shader attributes and uniforms
//======================================================================================
#ifdef WR_FRAGMENT_SHADER
    uniform sampler2D sTiling;
    uniform vec4 uInitialColor;
#endif

//======================================================================================
// Shared uniforms
//======================================================================================
//uniform int uCommandCount;

//======================================================================================
// Interpolator definitions
//======================================================================================

varying vec3 vPos0;
flat varying vec4 vColor0;
flat varying vec4 vRect0;

varying vec3 vPos1;
flat varying vec4 vColor1;
flat varying vec4 vRect1;

varying vec3 vPos2;
flat varying vec4 vColor2;
flat varying vec4 vRect2;

varying vec4 vGenericPos[6];
flat varying uvec4 vGenericColor[2];
flat varying vec4 vGenericRect[6];

varying vec2 vCompositeUv0;
varying vec2 vCompositeUv1;

//======================================================================================
// Shared types and constants
//======================================================================================
#define PRIM_KIND_RECT      uint(0)
#define PRIM_KIND_IMAGE     uint(1)
#define PRIM_KIND_TEXT      uint(3)
#define PRIM_KIND_INVALID   uint(4)

//======================================================================================
// Shared types and UBOs
//======================================================================================

//======================================================================================
// VS only types and UBOs
//======================================================================================
#ifdef WR_VERTEX_SHADER

struct TilePrimitive {
    vec4 rect;
    vec4 st;
    vec4 color;
};

struct CompositeTile {
    uvec4 rect;
    vec4 uv_rect0;
    vec4 uv_rect1;
};

layout(std140) uniform Tiles_Composite {
    CompositeTile tiles_composite[32];
};

struct EmptyTile {
    uvec4 rect;
};

layout(std140) uniform Tiles_Empty {
    EmptyTile tiles_empty[32];
};

struct L4P1Tile {
    uvec4 target_rect;
    uvec4 screen_rect;
    uvec4 layer_info;
    uvec4 prim_info;
    TilePrimitive prim;
};

layout(std140) uniform Tiles_L4P1 {
    L4P1Tile tiles_l4p1[128];
};

struct L4P2Tile {
    uvec4 target_rect;
    uvec4 screen_rect;
    uvec4 layer_info;
    uvec4 prim_info;
    TilePrimitive prims[2];
};

layout(std140) uniform Tiles_L4P2 {
    L4P2Tile tiles_l4p2[128];
};

struct L4P3Tile {
    uvec4 target_rect;
    uvec4 screen_rect;
    uvec4 layer_info;
    uvec4 prim_info;
    TilePrimitive prims[3];
};

layout(std140) uniform Tiles_L4P3 {
    L4P3Tile tiles_l4p3[128];
};

struct L4P4Tile {
    uvec4 target_rect;
    uvec4 screen_rect;
    uvec4 layer_info;
    uvec4 prim_info;
    TilePrimitive prims[4];
};

layout(std140) uniform Tiles_L4P4 {
    L4P4Tile tiles_l4p4[128];
};

struct L4P6Tile {
    uvec4 target_rect;
    uvec4 screen_rect;
    uvec4 layer_info0;
    uvec4 layer_info1;
    uvec4 prim_info0;
    uvec4 prim_info1;
    TilePrimitive prims[6];
};

layout(std140) uniform Tiles_L4P6 {
    L4P6Tile tiles_l4p6[128];
};

struct Layer {
    mat4 inv_transform;
    vec4 screen_vertices[4];
};

layout(std140) uniform Layers {
    Layer layers[32];
};

#endif

//======================================================================================
// Shared functions
//======================================================================================

//======================================================================================
// VS only functions
//======================================================================================
#ifdef WR_VERTEX_SHADER

vec2 write_vertex(vec4 target_rect, vec4 screen_rect) {
    vec4 actual_pos = vec4(target_rect.xy + aPosition.xy * target_rect.zw, 0, 1);
    vec2 virtual_pos = screen_rect.xy + aPosition.xy * screen_rect.zw;
    gl_Position = uTransform * actual_pos;
    return virtual_pos;
}

uint pack_color(vec4 color) {
    uint r = uint(color.r * 255.0) <<  0;
    uint g = uint(color.g * 255.0) <<  8;
    uint b = uint(color.b * 255.0) << 16;
    uint a = uint(color.a * 255.0) << 24;
    return r | g | b | a;
}

bool ray_plane(vec3 normal, vec3 point, vec3 ray_origin, vec3 ray_dir, out float t)
{
    float denom = dot(normal, ray_dir);
    if (denom > 1e-6) {
        vec3 d = point - ray_origin;
        t = dot(d, normal) / denom;
        return t >= 0.0;
    }

    return false;
}

vec4 untransform(vec2 ref, vec3 n, vec3 a, mat4 inv_transform) {
    vec3 p = vec3(ref, -100.0);
    vec3 d = vec3(0, 0, 1.0);

    float t;
    ray_plane(n, a, p, d, t);
    vec3 c = p + d * t;

    vec4 r = inv_transform * vec4(c, 1.0);
    return vec4(r.xyz / r.w, r.w);
}

vec3 get_layer_pos(vec2 pos, uint layer_index) {
    Layer layer = layers[layer_index];
    vec3 a = layer.screen_vertices[0].xyz / layer.screen_vertices[0].w;
    vec3 b = layer.screen_vertices[3].xyz / layer.screen_vertices[3].w;
    vec3 c = layer.screen_vertices[2].xyz / layer.screen_vertices[2].w;
    vec3 n = normalize(cross(b-a, c-a));
    vec4 local_pos = untransform(pos, n, a, layer.inv_transform);
    return local_pos.xyw;
}

#endif

//======================================================================================
// FS only functions
//======================================================================================
#ifdef WR_FRAGMENT_SHADER

vec4 fetch_initial_color() {
    return uInitialColor;
}

vec4 unpack_color(uint color) {
    float r = float(color & uint(0x000000ff)) / 255.0;
    float g = float((color & uint(0x0000ff00)) >> 8) / 255.0;
    float b = float((color & uint(0x00ff0000)) >> 16) / 255.0;
    float a = float((color & uint(0xff000000)) >> 24) / 255.0;
    return vec4(r, g, b, a);
}

float inside_box(vec2 p, vec2 p0, vec2 p1) {
    vec2 s = step(p0, p) - step(p1, p);
    return s.x * s.y;
}

bool point_in_rect(vec2 p, vec2 p0, vec2 p1) {
    return p.x >= p0.x &&
           p.y >= p0.y &&
           p.x <= p1.x &&
           p.y <= p1.y;
}

#endif
