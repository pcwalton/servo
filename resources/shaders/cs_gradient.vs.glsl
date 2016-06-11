/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct CachedGradient {
    ivec4 rect;
    vec4 local_rect;
    ivec4 offset_layer_dir;
    vec4 color0;
    vec4 color1;
};

layout(std140) uniform Items {
    CachedGradient gradients[512];
};

void main(void)
{
    CachedGradient gradient = gradients[gl_InstanceID];

    vec2 virtual_pos = mix(gradient.offset_layer_dir.xy,
                           gradient.offset_layer_dir.xy + gradient.rect.zw,
                           aPosition.xy);
    virtual_pos /= uDevicePixelRatio;
    vec3 layer_pos = get_layer_pos(virtual_pos, uint(gradient.offset_layer_dir.z));

    vColor0 = gradient.color0;
    vColor1 = gradient.color1;
    vDir = gradient.offset_layer_dir.w;
    vFraction = (layer_pos.xy - gradient.local_rect.xy) / gradient.local_rect.zw;

    vec2 pos = mix(gradient.rect.xy, gradient.rect.xy + gradient.rect.zw, aPosition.xy);
    gl_Position = uTransform * vec4(pos, 0, 1);
}
