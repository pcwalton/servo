IN_ATTRIBUTE vec3 aPosition;
IN_ATTRIBUTE vec4 aColor;

uniform mat4 uTransform;

OUT_VARYING vec4 vColor;
OUT_VARYING vec2 vPosition;

void main(void)
{
	vColor = aColor;
	vPosition = aPosition.xy;
    gl_Position = uTransform * vec4(aPosition, 1.0);
}
