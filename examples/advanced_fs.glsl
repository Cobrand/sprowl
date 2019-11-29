#version 330 core
in vec2 TexCoords;

out vec4 color;

uniform sampler2D texture0;
uniform sampler2D texture1;
uniform sampler2D texture2;
uniform sampler2D texture3;
uniform sampler2D texture4;
uniform sampler2D texture5;
uniform sampler2D texture6;
uniform sampler2D texture7;
uniform sampler2D texture8;
uniform sampler2D texture9;

uniform vec2 outline_thickness;
uniform vec4 outline_color;
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

    color = true_tex_color(texture0, TexCoords);
    if (effect == uint(3)) {
        vec2 noisecoords = vec2(
            TexCoords.x - (5.0 * cos(t / 30.0) + t) / 1024.0,
            TexCoords.y - (5.0 * sin(t / 20.0) + t) / 1024.0
        );
        vec3 noise = texture(texture1, noisecoords).rgb;
        const vec3 gold = vec3(ivec3(255, 208, 0)) / 255.0;
        color.rgb = mix(effect_color.rgb, gold, vec3(noise.g, noise.b, (noise.g+noise.b) / 2.0));
    }
    if (outline_color.a == 0.0) {
        return;
    }
    float v = get_border_alpha(texture0, TexCoords, outline_thickness);
    if (v > 0.0) {
        color = blend(color, vec4(outline_color.rgb, v));
    }
}