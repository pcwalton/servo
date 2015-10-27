IN_ATTRIBUTE vec2 aPosition;
IN_ATTRIBUTE vec4 aColor;

uniform mat4 uTransform;

OUT_VARYING vec4 vColor;

void main(void)
{
	vColor = aColor / 255.0;
    gl_Position = uTransform * vec4(aPosition, 0.0, 1.0);
}
