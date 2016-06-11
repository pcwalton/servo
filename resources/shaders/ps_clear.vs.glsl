/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main() {
    Tile tile = tiles[gl_InstanceID];

    vec4 rect = tile.rect; // unpack_au_rect(tile.rect);

    vec4 pos = vec4(mix(rect.xy, rect.xy + rect.zw, aPosition.xy), 0, 1);
    gl_Position = uTransform * pos;
}
