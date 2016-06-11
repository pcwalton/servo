/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define MAX_PRIMS_PER_COMPOSITE         (8)

#define INVALID_LAYER_INDEX             uint(0xffffffff)

uniform sampler2D sLayer0;
uniform sampler2D sLayer1;
uniform sampler2D sLayer2;
uniform sampler2D sLayer3;
uniform sampler2D sLayer4;
uniform sampler2D sLayer5;
uniform sampler2D sLayer6;
uniform sampler2D sLayer7;
uniform sampler2D sCache;

struct CompositeTile {
    ivec4 rect;
    uvec4 prim_indices[MAX_PRIMS_PER_COMPOSITE/4];
    uvec4 layer_indices[MAX_PRIMS_PER_COMPOSITE/4];
};

struct Layer {
    mat4 transform;
    mat4 inv_transform;
    vec4 screen_vertices[4];
    vec4 blend_info;
};

layout(std140) uniform Layers {
    Layer layers[256];
};

layout(std140) uniform Tiles {
    CompositeTile tiles[384];
};

struct Primitive {
    ivec4 screen_rect;
    vec4 st_rect;
};

layout(std140) uniform Primitives {
    Primitive primitives[585];
};

#ifdef WR_VERTEX_SHADER

vec2 write_prim(vec2 pos, uint prim_index) {
    Primitive prim = primitives[prim_index];
    vec4 prim_rect = prim.screen_rect;
    vec2 f = (pos - prim_rect.xy) / prim_rect.zw;
    return mix(prim.st_rect.xy, prim.st_rect.zw, f);
}

vec2 write_vertex(CompositeTile tile) {
    vec4 pos = vec4(mix(tile.rect.xy,
                        tile.rect.xy + tile.rect.zw,
                        aPosition.xy),
                    0.0,
                    1.0);
    gl_Position = uTransform * pos;
    return pos.xy;
}

#endif

#ifdef WR_FRAGMENT_SHADER

vec4 fetch_initial_color() {
    return vec4(1, 1,1, 1);
}

#endif
