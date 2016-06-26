#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

uniform vec2 uFramebufferSize;

in vec3 aPosition;
in vec2 aRenderableId;

out vec2 vRenderableId;

void main() {
    vRenderableId = aRenderableId;
    int vertex = gl_VertexID % 6;
    vec2 roundedPosition =
        vec2((which == 0 || which == 2 || which == 5) ? floor(position.x) : ceil(position.x),
             (which == 0 || which == 1 || which == 3) ? floor(position.y) : ceil(position.y));
    gl_Position = vec4(roundedPosition / uFramebufferSize * 2.0 - 1.0, aPosition.z / 4096.0, 1.0);
}

