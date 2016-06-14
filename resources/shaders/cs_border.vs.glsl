/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Border {
    ivec4 screen_origin_layer_unused;
    ivec4 screen_rect;
    vec4 local_rect;
    vec4 left_color;
    vec4 top_color;
    vec4 right_color;
    vec4 bottom_color;
    vec4 widths;
    Clip clip;
};

layout(std140) uniform Items {
    Border borders[128];
};

void main(void) {
    Border border = borders[gl_InstanceID];

    vec2 pos = mix(border.screen_rect.xy, border.screen_rect.xy + border.screen_rect.zw, aPosition.xy);
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);

    vec2 virtual_pos = mix(border.screen_origin_layer_unused.xy,
                           border.screen_origin_layer_unused.xy + border.screen_rect.zw,
                           aPosition.xy);
    virtual_pos /= uDevicePixelRatio;

    vLayerPos = get_layer_pos(virtual_pos, uint(border.screen_origin_layer_unused.z));
    vLeftColor = border.left_color;
    vRightColor = border.right_color;
    vTopColor = border.top_color;
    vBottomColor = border.bottom_color;

    vec4 widths = vec4(max(border.clip.top_left.outer_inner_radius.x, border.widths.x),
                       max(border.clip.top_left.outer_inner_radius.x, border.widths.y),
                       max(border.clip.top_left.outer_inner_radius.x, border.widths.z),
                       max(border.clip.top_left.outer_inner_radius.x, border.widths.w));

    vCorner_TL = border.local_rect.xy + widths.xy;
    vCorner_TR = border.local_rect.xy + vec2(border.local_rect.z, 0) + vec2(-widths.z, widths.y);
    vCorner_BL = border.local_rect.xy + vec2(0, border.local_rect.w) + vec2(widths.x, -widths.w);
    vCorner_BR = border.local_rect.xy + border.local_rect.zw - widths.zw;
    vWidths = border.widths;
    vRect = vec4(border.local_rect.xy, border.local_rect.xy + border.local_rect.zw);

    vClipRect = vec4(border.clip.rect.xy, border.clip.rect.xy + border.clip.rect.zw);
    vClipInfo = border.clip.top_left.outer_inner_radius;
}
