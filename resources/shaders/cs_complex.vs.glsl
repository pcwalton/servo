/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct CachedPrimitive {
    ivec4 rect;
    ivec4 offset_layer;
    vec4 color;
    Clip clip;
};

layout(std140) uniform Items {
    CachedPrimitive prims[256];
};

void main(void)
{
    CachedPrimitive prim = prims[gl_InstanceID];

    vec2 pos = mix(prim.rect.xy, prim.rect.xy + prim.rect.zw, aPosition.xy);
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);

    vec2 virtual_pos = mix(prim.offset_layer.xy,
                           prim.offset_layer.xy + prim.rect.zw,
                           aPosition.xy);
    virtual_pos /= uDevicePixelRatio;

    vLayerPos = get_layer_pos(virtual_pos, uint(prim.offset_layer.z));
    vClipRect = vec4(prim.clip.rect.xy, prim.clip.rect.xy + prim.clip.rect.zw);
    vClipInfo = prim.clip.top_left.outer_inner_radius;

    vColor = prim.color;
}
