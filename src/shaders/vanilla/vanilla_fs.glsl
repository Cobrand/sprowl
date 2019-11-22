#version 330 core
in vec2 TexCoords;

out vec4 color;

uniform sampler2D image;
uniform uint is_grayscale;

void main()
{
    color = texture(image, TexCoords);
    if (is_grayscale == uint(1)) {
        color.a = color.r;
        color.rgb = vec3(1.0);
    }
}