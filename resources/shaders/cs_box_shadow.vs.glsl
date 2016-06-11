/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define BS_CLIP_NONE    0
#define BS_CLIP_OUTSET  1
#define BS_CLIP_INSET   2

struct CachedBoxShadow {
    ivec4 rect;
    ivec4 offset_layer_clip_inverted;
    vec4 color;
    vec4 border_radii_blur_radius;
    vec4 bs_rect;
    vec4 src_rect;
};

layout(std140) uniform Items {
    CachedBoxShadow box_shadows[256];
};

void main(void)
{
    CachedBoxShadow bs = box_shadows[gl_InstanceID];

    vec2 virtual_pos = mix(bs.offset_layer_clip_inverted.xy,
                           bs.offset_layer_clip_inverted.xy + bs.rect.zw,
                           aPosition.xy);
    virtual_pos /= uDevicePixelRatio;

    vLayerPos = get_layer_pos(virtual_pos, uint(bs.offset_layer_clip_inverted.z));
    vColor = bs.color;
    vBorderRadii = bs.border_radii_blur_radius.xy;
    vBlurRadius = bs.border_radii_blur_radius.z;
    vBoxShadowRect = vec4(bs.bs_rect.xy, bs.bs_rect.xy + bs.bs_rect.zw);
    vSrcRect = vec4(bs.src_rect.xy, bs.src_rect.xy + bs.src_rect.zw);
    vInverted = bs.offset_layer_clip_inverted.w;

    vec2 pos = mix(bs.rect.xy, bs.rect.xy + bs.rect.zw, aPosition.xy);
    gl_Position = uTransform * vec4(pos, 0, 1);
}
