/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct CachedRectangle {
    uvec4 rect;
    vec4 color;
};

layout(std140) uniform Items {
    CachedRectangle rects[1024];
};

void main(void)
{
    CachedRectangle rect = rects[gl_InstanceID];

    vec2 pos = mix(rect.rect.xy, rect.rect.xy + rect.rect.zw, aPosition.xy);
    gl_Position = uTransform * vec4(pos, 0.0, 1.0);

    vColor = rect.color;
}
