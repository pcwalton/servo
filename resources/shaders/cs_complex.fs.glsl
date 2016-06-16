/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    vec2 pos = vLayerPos.xy / vLayerPos.z;

    if (point_in_rect(pos, vClipRect.xy, vClipRect.zw)) {
        float clip_mask = do_clip(pos, vClipRect, vClipInfo.x);
        oFragColor = clip_mask * vColor;
    } else {
        discard;
    }
}
