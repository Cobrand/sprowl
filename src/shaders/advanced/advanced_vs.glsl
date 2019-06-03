#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tex_coords;

out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;

void main()
{
    TexCoords = tex_coords;
    gl_Position = view * model * vec4(pos, 0.0, 1.0);
}