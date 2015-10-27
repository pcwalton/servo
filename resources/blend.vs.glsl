IN_ATTRIBUTE vec3 aPosition;
IN_ATTRIBUTE vec2 aColorTexCoord;
IN_ATTRIBUTE vec2 aMaskTexCoord;

uniform mat4 uTransform;

OUT_VARYING vec2 vColorTexCoord;
OUT_VARYING vec2 vMaskTexCoord;

void main(void)
{
	vColorTexCoord = aColorTexCoord / 65535.0;
	vMaskTexCoord = aMaskTexCoord / 65535.0;
    gl_Position = uTransform * vec4(aPosition, 1.0);
}
