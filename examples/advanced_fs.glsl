#version 330 core
in vec2 TexCoords;

out vec4 color;

uniform vec2 outline_thickness;
uniform vec4 outline_color;
uniform vec4 background_color;
uniform sampler2D img;
uniform uint effect;
uniform float t;
uniform uint is_grayscale;

vec4 true_tex_color(sampler2D img, vec2 pos) {
    vec4 color = texture(img, pos);
    if (is_grayscale == uint(1)) {
        color.a = color.r;
        color.rgb = vec3(1.0);
    }
    return color;
}

void main()
{
    if (effect == uint(1)) {
        // glowing effect
        const vec4 base_color = vec4(ivec4(247, 118, 34, 255)) / 255.0;
        const vec4 half_base_color = base_color / 2.0;
        color = half_base_color * vec4(
            cos(- 10.0 * TexCoords.x) * sin(10.0 * TexCoords.y) * cos(t / 10.0),
            cos( 10.0 * TexCoords.x) * sin(10.0 * TexCoords.y),
            cos( 10.0 * TexCoords.x) * sin(- 10.0 * TexCoords.y),
            1.0
        ) + half_base_color;
        return;
    }
    if (effect == uint(2)) {
        // solid colored shape
        color = background_color;
        return;
    }

    float v = 0.0;
    if (outline_color.a == 0.0) {
        color = true_tex_color(img, TexCoords);
        return;
    }
    v += ceil(true_tex_color(img, vec2(TexCoords.x - outline_thickness.x, TexCoords.y - outline_thickness.y)).a);
    v += ceil(true_tex_color(img, vec2(TexCoords.x                      , TexCoords.y - outline_thickness.y)).a);
    v += ceil(true_tex_color(img, vec2(TexCoords.x + outline_thickness.x, TexCoords.y - outline_thickness.y)).a);

    v += ceil(true_tex_color(img, vec2(TexCoords.x - outline_thickness.x, TexCoords.y)).a);
    // center alpha
    float c_a = texture(img, TexCoords).a;
    v += ceil(true_tex_color(img, vec2(TexCoords.x + outline_thickness.x, TexCoords.y)).a);

    v += ceil(true_tex_color(img, vec2(TexCoords.x - outline_thickness.x, TexCoords.y + outline_thickness.y)).a);
    v += ceil(true_tex_color(img, vec2(TexCoords.x                      , TexCoords.y + outline_thickness.y)).a);
    v += ceil(true_tex_color(img, vec2(TexCoords.x + outline_thickness.x, TexCoords.y + outline_thickness.y)).a);
    if (c_a <= 0.0 && v > 0.0) {
        color = outline_color;
    } else if (c_a <= 0.0) {
        color = background_color;
    } else {
        color = true_tex_color(img, TexCoords);
    }
}