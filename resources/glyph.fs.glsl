#ifdef GL_ES
    precision mediump float;
#endif

#ifdef HAVE_TEXTURE_ARRAY
    uniform sampler2DArray sDiffuse;
#else
    uniform sampler2D sDiffuse;
#endif

IN_VARYING vec4 vColor;

#ifdef HAVE_TEXTURE_ARRAY
    IN_VARYING vec3 vTexCoord;
#else
    IN_VARYING vec2 vTexCoord;
#endif

DEFINE_FRAG_COLOR_OUTPUT;

void main(void)
{
    vec4 diffuse = TEXTURE(sDiffuse, vTexCoord);

	#ifdef PLATFORM_ANDROID
		float alpha = diffuse.a;
	#else
		float alpha = diffuse.r;
	#endif

	FRAG_COLOR = vec4(vColor.xyz, alpha * vColor.w);
}
