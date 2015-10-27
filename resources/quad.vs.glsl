IN_ATTRIBUTE vec3 aPosition;
IN_ATTRIBUTE vec2 aColorTexCoord;
IN_ATTRIBUTE vec2 aMaskTexCoord;
IN_ATTRIBUTE vec4 aColor;
IN_ATTRIBUTE vec4 aMatrixIndex;

uniform mat4 uTransform;
uniform mat4 uMatrixPalette[32];
uniform float uDevicePixelRatio;

OUT_VARYING vec4 vColor;
OUT_VARYING vec3 vColorTexCoord;
OUT_VARYING vec3 vMaskTexCoord;

#ifdef HAVE_TEXTURE_ARRAY
    IN_ATTRIBUTE float aColorTexIndex;
    IN_ATTRIBUTE float aMaskTexIndex;
#endif

void main(void)
{
    vColor = aColor / 255.0;

    #ifdef HAVE_TEXTURE_ARRAY
        vColorTexCoord = vec3(aColorTexCoord.xy / 65535.0, aColorTexIndex);
        vMaskTexCoord = vec3(aMaskTexCoord.xy / 65535.0, aMaskTexIndex);
    #else
        vColorTexCoord = aColorTexCoord / 65535.0;
        vMaskTexCoord = aMaskTexCoord / 65535.0;
    #endif

    mat4 matrix = uMatrixPalette[int(aMatrixIndex.x)];
    vec4 pos = matrix * vec4(aPosition, 1.0);
    pos.xy = floor(pos.xy * uDevicePixelRatio + 0.5) / uDevicePixelRatio;
    gl_Position = uTransform * pos;
}
