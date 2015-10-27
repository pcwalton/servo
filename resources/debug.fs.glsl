#ifdef GL_ES
    precision mediump float;
#endif

IN_VARYING vec4 vColor;

DEFINE_FRAG_COLOR_OUTPUT;

void main(void)
{
	FRAG_COLOR = vColor;
}
