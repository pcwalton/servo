#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    vec4 prim_colors[6];
    prim_colors[0] = texture(sLayer0, vUv0);
    prim_colors[1] = texture(sLayer1, vUv1);
    prim_colors[2] = texture(sLayer2, vUv2);
    prim_colors[3] = texture(sLayer3, vUv3);
    prim_colors[4] = texture(sLayer4, vUv4);
    prim_colors[5] = texture(sLayer5, vUv5);

    vec4 result = vec4(1, 1, 1, 1);
    vec4 layer_color = vec4(0, 0, 0, 0);

    layer_color = mix(layer_color, prim_colors[0], prim_colors[0].a);
    result = mix(result, layer_color, layer_color.a * vLayerValues0.x);
    layer_color = mix(layer_color, vec4(0, 0, 0, 0), vec4(vLayerValues0.x > 0.0));

    layer_color = mix(layer_color, prim_colors[1], prim_colors[1].a);
    result = mix(result, layer_color, layer_color.a * vLayerValues0.y);
    layer_color = mix(layer_color, vec4(0, 0, 0, 0), vec4(vLayerValues0.y > 0.0));

    layer_color = mix(layer_color, prim_colors[2], prim_colors[2].a);
    result = mix(result, layer_color, layer_color.a * vLayerValues0.z);
    layer_color = mix(layer_color, vec4(0, 0, 0, 0), vec4(vLayerValues0.z > 0.0));

    layer_color = mix(layer_color, prim_colors[3], prim_colors[3].a);
    result = mix(result, layer_color, layer_color.a * vLayerValues0.w);
    layer_color = mix(layer_color, vec4(0, 0, 0, 0), vec4(vLayerValues0.w > 0.0));

    layer_color = mix(layer_color, prim_colors[4], prim_colors[4].a);
    result = mix(result, layer_color, layer_color.a * vLayerValues1.x);
    layer_color = mix(layer_color, vec4(0, 0, 0, 0), vec4(vLayerValues1.x > 0.0));

    layer_color = mix(layer_color, prim_colors[5], prim_colors[5].a);
    result = mix(result, layer_color, layer_color.a * vLayerValues1.y);

    oFragColor = result;
}
