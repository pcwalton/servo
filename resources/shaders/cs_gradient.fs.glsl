/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define DIR_HORIZONTAL  0
#define DIR_VERTICAL    1

void main(void)
{
    float f;

    switch (vDir)
    {
        case DIR_HORIZONTAL:
            f = vFraction.x;
            break;
        case DIR_VERTICAL:
            f = vFraction.y;
            break;
    }

    oFragColor = mix(vColor0, vColor1, f);
}
