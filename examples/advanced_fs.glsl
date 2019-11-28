#version 330 core
in vec2 TexCoords;

out vec4 color;

uniform vec2 outline_thickness;
uniform vec4 outline_color;
uniform sampler2D img;
uniform uint effect;
uniform vec4 effect_color;
uniform float t;
uniform uint is_grayscale;

vec4 blend(vec4 src, vec4 dst) {
    return src * vec4(src.a) + dst * vec4(1.0 - src.a);
}

vec4 true_tex_color(sampler2D img, vec2 pos) {
    vec4 color = texture(img, pos);
    if (is_grayscale == uint(1)) {
        color.a = color.r;
        color.rgb = vec3(1.0);
    }
    return color;
}

float get_border_alpha(sampler2D img, vec2 pos, vec2 outline_thickness) {
    float v = 0.0;
    v = max(v, true_tex_color(img, vec2(TexCoords.x - outline_thickness.x, TexCoords.y - outline_thickness.y)).a);
    v = max(v, true_tex_color(img, vec2(TexCoords.x                      , TexCoords.y - outline_thickness.y)).a);
    v = max(v, true_tex_color(img, vec2(TexCoords.x + outline_thickness.x, TexCoords.y - outline_thickness.y)).a);

    v = max(v, true_tex_color(img, vec2(TexCoords.x - outline_thickness.x, TexCoords.y                      )).a);
    v = max(v, true_tex_color(img, vec2(TexCoords.x + outline_thickness.x, TexCoords.y                      )).a);

    v = max(v, true_tex_color(img, vec2(TexCoords.x - outline_thickness.x, TexCoords.y + outline_thickness.y)).a);
    v = max(v, true_tex_color(img, vec2(TexCoords.x                      , TexCoords.y + outline_thickness.y)).a);
    v = max(v, true_tex_color(img, vec2(TexCoords.x + outline_thickness.x, TexCoords.y + outline_thickness.y)).a);
    return v;
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
        color = effect_color;
        return;
    }

    color = true_tex_color(img, TexCoords);
    if (outline_color.a == 0.0) {
        return;
    }
    float v = get_border_alpha(img, TexCoords, outline_thickness);
    if (v > 0.0) {
        color = blend(color, vec4(outline_color.rgb, v));
    }
}