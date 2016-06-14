/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

bool point_above_line(vec2 p, vec2 p0, vec2 p1) {
    return (p.x - p0.x) * (p1.y - p0.y) - (p.y - p0.y) * (p1.x - p0.x) > 0.0;
}

bool do_corner_clip(vec2 pos, vec2 ref, float outer_radius, float inner_radius) {
    float d = distance(pos, ref);

    return outer_radius > 0.0 && (d > outer_radius || d < inner_radius);
}

void main(void) {
    // TODO(gw): Check compiled GLSL assembly and see if this
    //           gets turned into something reasonable...

    // TODO(gw): This shader handles cases where each border
    //           width is different. It's probably really inefficient
    //           for the common case of equal border widths.
    //           Investigate a fast path for this case!

    bool in_top = point_above_line(vLayerPos.xy, vCorner_TL, vCorner_TR);
    bool in_bottom = point_above_line(vLayerPos.xy, vCorner_BR, vCorner_BL);
    bool in_left = point_above_line(vLayerPos.xy, vCorner_BL, vCorner_TL);
    bool in_right = point_above_line(vLayerPos.xy, vCorner_TR, vCorner_BR);

    if (in_top && in_left) {
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
    } else if (in_top && in_right) {
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
    } else if (in_left && in_bottom) {
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
    } else if (in_right && in_bottom) {
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
    } else if (in_top) {
        oFragColor = vTopColor;
    } else if (in_left) {
        oFragColor = vLeftColor;
    } else if (in_right) {
        oFragColor = vRightColor;
    } else if (in_bottom) {
        oFragColor = vBottomColor;
    } else {
        discard;
    }
}
