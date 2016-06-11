#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main() {
    CompositeTile tile = tiles[gl_InstanceID];
    vec2 pos = write_vertex(tile);

    vUv0 = write_prim(pos, tile.prim_indices[0].x);
    vUv1 = write_prim(pos, tile.prim_indices[0].y);

    uint li0 = tile.layer_indices[0].x;
    uint li1 = tile.layer_indices[0].y;

    if (li0 == INVALID_LAYER_INDEX || li0 == li1) {
        vLayerValues.x = 0.0;
    } else {
        vLayerValues.x = layers[li0].blend_info.x;
    }

    if (li1 == INVALID_LAYER_INDEX) {
        vLayerValues.y = 0.0;
    } else {
        vLayerValues.y = layers[li1].blend_info.x;
    }
}
