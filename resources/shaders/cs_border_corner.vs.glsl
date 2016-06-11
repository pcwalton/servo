/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct CachedBorderCorner {
    vec4 rect;
    vec4 color0;
    vec4 color1;
    uvec4 layer_corner_kind;
    vec4 screen_origin_offset;
    //vec4 outer_inner_radius;
};

layout(std140) uniform Items {
    CachedBorderCorner corners[512];
};

void main(void)
{
    CachedBorderCorner corner = corners[gl_InstanceID];

    vec2 local_pos = mix(corner.rect.xy, corner.rect.xy + corner.rect.zw, aPosition.xy);
    gl_Position = uTransform * vec4(local_pos, 0, 1);

    vec2 virtual_pos = (local_pos - corner.screen_origin_offset.xy) / uDevicePixelRatio;
    virtual_pos += corner.screen_origin_offset.zw;

    vec3 layer_pos = get_layer_pos(virtual_pos, corner.layer_corner_kind.x);

    //Layer layer = layers[corner.layer_corner_kind.x];
    //vec4 pos = layer.transform * vec4(local_pos, 0.0, 1.0);
    //pos.xy -= corner.screen_origin_offset.xy;
    //pos.xy = pos.xy * uDevicePixelRatio + corner.screen_origin_offset.zw;

    vDebug = layer_pos.xy * 0.01;// aPosition.xy;

/*
    vClipRadii = corner.outer_inner_radius;
    vClipPos = local_pos;
    vClipRef = corner.rect.xy;
*/

    vColor0 = corner.color0;
    vColor1 = corner.color1;
    vInfo = aPosition.xy;

    switch (corner.layer_corner_kind.y) {
        case CORNER_TOP_LEFT:
            //vClipRef += corner.outer_inner_radius.xy;
            //vClipSign = vec2(1, 1);
            break;
        case CORNER_TOP_RIGHT:
            //vClipRef.x += corner.rect.z;
            //vClipRef.x -= corner.outer_inner_radius.x;
            //vClipRef.y += corner.outer_inner_radius.y;
            vInfo.x = 1.0 - vInfo.x;
            //vClipSign = vec2(-1, 1);
            break;
        case CORNER_BOTTOM_LEFT:
            //vClipRef.y += corner.rect.w;
            //vClipRef.x += corner.outer_inner_radius.x;
            //vClipRef.y -= corner.outer_inner_radius.y;
            vInfo.y = 1.0 - vInfo.y;
            //vClipSign = vec2(1, -1);
            break;
        case CORNER_BOTTOM_RIGHT:
            //vClipRef += corner.rect.zw;
            //vClipRef -= corner.outer_inner_radius.xy;
            vInfo.x = 1.0 - vInfo.x;
            vInfo.y = 1.0 - vInfo.y;
            //vClipSign = vec2(-1, -1);
            break;
    }
}
