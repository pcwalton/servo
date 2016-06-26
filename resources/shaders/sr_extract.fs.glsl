#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

uniform sampler2DMS uTexture;
uniform int uSamples;

out vec4 oFragColor;

void main() {
    ivec2 fragCoord = ivec2(int(gl_FragCoord.x - 0.5), int(gl_FragCoord.y - 0.5));
    ivec2 uv = ivec2(fragCoord.x / uSamples, fragCoord.y);
    int sampleIndex = fragCoord.x % uSamples;
    oFragColor = texelFetch(uTexture, uv, sampleIndex);
}

