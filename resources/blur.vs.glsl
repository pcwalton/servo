IN_ATTRIBUTE vec3 aPosition;
IN_ATTRIBUTE vec2 aMaskTexCoord;

uniform mat4 uTransform;

OUT_VARYING vec2 vMaskTexCoord;

void main(void)
{
	vMaskTexCoord = aMaskTexCoord;
    gl_Position = uTransform * vec4(aPosition, 1.0);
}

