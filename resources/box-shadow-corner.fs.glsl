#ifdef GL_ES
    precision mediump float;
#endif

uniform sampler2D sDiffuse;

uniform vec4 uPosition;
uniform float uBlurRadius;
uniform float uArcRadius;

IN_VARYING vec4 vColor;
IN_VARYING vec2 vPosition;

DEFINE_FRAG_COLOR_OUTPUT;

void main(void)
{
    vec2 lPosition = vPosition - uPosition.xy;
    vec2 lArcCenter = uPosition.zw;
    float lDistance = distance(lPosition, vec2(lArcCenter));
    float lValue = clamp(lDistance, uArcRadius - uBlurRadius, uArcRadius + uBlurRadius);
    lValue = ((lValue - uArcRadius) / uBlurRadius + 1.0) / 2.0;
    FRAG_COLOR = vec4(1.0 - lValue);
}

