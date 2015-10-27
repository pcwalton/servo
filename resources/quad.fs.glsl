#ifdef GL_ES
    precision mediump float;
#endif

#ifdef HAVE_TEXTURE_ARRAY
    uniform sampler2DArray sDiffuse;
    uniform sampler2DArray sMask;
#else
    uniform sampler2D sDiffuse;
    uniform sampler2D sMask;
#endif

IN_VARYING vec4 vColor;

#ifdef HAVE_TEXTURE_ARRAY
    IN_VARYING vec3 vColorTexCoord;
    IN_VARYING vec3 vMaskTexCoord;
#else
    IN_VARYING vec2 vColorTexCoord;
    IN_VARYING vec2 vMaskTexCoord;
#endif

DEFINE_FRAG_COLOR_OUTPUT;

void main(void)
{
    #ifdef HAVE_TEXTURE_ARRAY
        vec4 diffuse = TEXTURE(sDiffuse, vColorTexCoord);
        vec4 mask = TEXTURE(sMask, vMaskTexCoord);
    #else
        vec4 diffuse = TEXTURE(sDiffuse, vColorTexCoord);
        vec4 mask = TEXTURE(sMask, vMaskTexCoord);
    #endif

	#ifdef PLATFORM_ANDROID
        float alpha = mask.a;
    #else
        float alpha = mask.r;
    #endif

	FRAG_COLOR = diffuse * vec4(vColor.rgb, vColor.a * alpha);
}
