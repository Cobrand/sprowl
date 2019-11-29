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
        const vec3 base_color = vec3(ivec3(247, 118, 34)) / 255.0;
        const vec3 half_base_color = base_color / 2.0;
        color.rgb = half_base_color * vec3(
            cos(- 10.0 * TexCoords.x) * sin(10.0 * TexCoords.y) * cos(t / 10.0),
            cos( 10.0 * TexCoords.x) * sin(10.0 * TexCoords.y),
            cos( 10.0 * TexCoords.x) * sin(- 10.0 * TexCoords.y)
        ) + half_base_color;
        return;
    }
    if (effect == uint(2)) {
        // solid colored shape
        color = effect_color;
        return;
    }

    color = true_tex_color(img, TexCoords);
    if (effect == uint(3)) {
        vec3 diff = (color.rgb - effect_color.rgb) / 4.0;
        color.rgb = effect_color.rgb + diff * vec3(
            cos(TexCoords.x * 20.0 * TexCoords.y * t),
            cos(TexCoords.x * 10.0 * TexCoords.y * t),
            cos(TexCoords.x * -40.0 * TexCoords.y * t)
        );
    }
    if (outline_color.a == 0.0) {
        return;
    }
    float v = get_border_alpha(img, TexCoords, outline_thickness);
    if (v > 0.0) {
        color = blend(color, vec4(outline_color.rgb, v));
    }
}