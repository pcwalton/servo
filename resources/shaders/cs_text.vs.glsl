/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Glyph {
    ivec4 offset_layer;
	vec4 color;
    vec4 p0_p1;
    vec4 st0_st1;
};

layout(std140) uniform Items {
    Glyph glyphs[1024];
};

void main(void)
{
    Glyph glyph = glyphs[gl_InstanceID];

    Layer layer = layers[glyph.offset_layer.z];

    vec2 local_pos = mix(glyph.p0_p1.xy, glyph.p0_p1.zw, aPosition.xy);
    vec4 pos = layer.transform * vec4(local_pos, 0.0, 1.0);

    pos.xy *= uDevicePixelRatio;

    float x = glyph.offset_layer.x;
    float y = glyph.offset_layer.y;
    mat4 xf;
    xf[0] = vec4(1, 0, 0, 0);
    xf[1] = vec4(0, 1, 0, 0);
    xf[2] = vec4(0, 0, 1, 0);
    xf[3] = vec4(x, y, 0, 1);

    //pos.xy += glyph.offset_layer.xy;

    gl_Position = uTransform * xf * pos;

    vUv = mix(glyph.st0_st1.xy, glyph.st0_st1.zw, aPosition.xy);
    vColor = glyph.color;
}

