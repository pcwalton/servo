/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void)
{
    /*
    float dist = distance(vClipPos, vClipRef);
    if (all(lessThan(vClipPos * vClipSign, vClipRef * vClipSign)) &&
        (dist > vClipRadii.x || dist < vClipRadii.z)) {
        discard;
    }*/

    float d = vInfo.x - vInfo.y;
    oFragColor = vec4(vDebug, 0, 1);//mix(vColor0, vColor1, step(0.0, d));
}
