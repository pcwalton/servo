/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void) {
    float clip_mask = do_clip(vLayerPos.xy, vClipRect, vClipInfo.x);
    oFragColor = clip_mask * vColor;
}
