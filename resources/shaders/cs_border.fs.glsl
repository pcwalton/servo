/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

bool point_above_line(vec2 p, vec2 p0, vec2 p1) {
    return (p.x - p0.x) * (p1.y - p0.y) - (p.y - p0.y) * (p1.x - p0.x) > 0.0;
}

void main(void) {
    // TODO(gw): Check compiled GLSL assembly and see if this
    //           gets turned into something reasonable...

    // TODO(gw): This shader handles cases where each border
    //           width is different. It's probably really inefficient
    //           for the common case of equal border widths.
    //           Investigate a fast path for this case!

    if (all(lessThan(vLayerPos.xy, vCorner_TL))) {
        vec2 ref = vClipRect.xy + vClipInfo.xy;
        if (vLayerPos.x < ref.x && vLayerPos.y < ref.y) {
            float d = distance(vLayerPos.xy, ref);
            if (d > vClipInfo.x || d < vClipInfo.z) {
                discard;
            }
        }

        if (point_above_line(vLayerPos.xy, vRect.xy, vCorner_TL)) {
            oFragColor = vTopColor;
        } else {
            oFragColor = vLeftColor;
        }
    } else if (vLayerPos.x > vCorner_TR.x && vLayerPos.y < vCorner_TR.y) {
        vec2 ref = vClipRect.zy + vec2(-vClipInfo.x, vClipInfo.y);
        if (vLayerPos.x > ref.x && vLayerPos.y < ref.y) {
            float d = distance(vLayerPos.xy, ref);
            if (d > vClipInfo.x || d < vClipInfo.z) {
                discard;
            }
        }

        if (point_above_line(vLayerPos.xy, vRect.zy, vCorner_TR)) {
            oFragColor = vRightColor;
        } else {
            oFragColor = vTopColor;
        }
    } else if (vLayerPos.x < vCorner_BL.x && vLayerPos.y > vCorner_BL.y) {
        vec2 ref = vClipRect.xw + vec2(vClipInfo.x, -vClipInfo.y);
        if (vLayerPos.x < ref.x && vLayerPos.y > ref.y) {
            float d = distance(vLayerPos.xy, ref);
            if (d > vClipInfo.x || d < vClipInfo.z) {
                discard;
            }
        }

        if (point_above_line(vLayerPos.xy, vRect.xw, vCorner_BL)) {
            oFragColor = vLeftColor;
        } else {
            oFragColor = vBottomColor;
        }
    } else if (all(greaterThan(vLayerPos.xy, vCorner_BR))) {
        vec2 ref = vClipRect.zw - vClipInfo.xy;
        if (vLayerPos.x > ref.x && vLayerPos.y > ref.y) {
            float d = distance(vLayerPos.xy, ref);
            if (d > vClipInfo.x || d < vClipInfo.z) {
                discard;
            }
        }

        if (point_above_line(vLayerPos.xy, vRect.zw, vCorner_BR)) {
            oFragColor = vBottomColor;
        } else {
            oFragColor = vRightColor;
        }
    } else if (vLayerPos.x < vRect.x + vWidths.x) {
        oFragColor = vLeftColor;
    } else if (vLayerPos.x > vRect.z - vWidths.z) {
        oFragColor = vRightColor;
    } else if (vLayerPos.y < vRect.y + vWidths.y) {
        oFragColor = vTopColor;
    } else if (vLayerPos.y > vRect.w - vWidths.w) {
        oFragColor = vBottomColor;
    } else {      
        discard;
    }
}
