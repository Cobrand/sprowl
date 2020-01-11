#version 330 core

in vec2 tex_coords;
// 0 = texture, 1 = text, 2 = shape,
flat in uint kind;
flat in uint layer;
flat in uint secondary_layer;
flat in uint effect;
in vec3 effect_color;

out vec4 color;

uniform sampler2DArray texture_rgba;
uniform sampler2DArray texture_gray;
uniform float t;

vec4 blend(vec4 src, vec4 dst) {
    return src * vec4(src.a) + dst * vec4(1.0 - src.a);
}

vec4 true_tex_color(sampler2DArray img, vec2 pos, float layer) {
    vec4 color = texture(img, vec3(pos, layer));
    if (kind == uint(1)) {
        color = vec4(
            1.0,
            1.0,
            1.0,
            color.r
        );
    }
    return color;
}

float get_border_alpha(sampler2DArray img, vec2 pos, float layer, vec2 outline_thickness) {
    float v = 0.0;
    v = max(v, true_tex_color(img, pos + outline_thickness * vec2(-1.0, -1.0), layer).a);
    v = max(v, true_tex_color(img, pos + outline_thickness * vec2( 0.0, -1.0), layer).a);
    v = max(v, true_tex_color(img, pos + outline_thickness * vec2( 1.0, -1.0), layer).a);

    v = max(v, true_tex_color(img, pos + outline_thickness * vec2(-1.0,  0.0), layer).a);
    v = max(v, true_tex_color(img, pos + outline_thickness * vec2( 1.0,  0.0), layer).a);

    v = max(v, true_tex_color(img, pos + outline_thickness * vec2(-1.0,  1.0), layer).a);
    v = max(v, true_tex_color(img, pos + outline_thickness * vec2( 0.0,  1.0), layer).a);
    v = max(v, true_tex_color(img, pos + outline_thickness * vec2( 1.0,  1.0), layer).a);
    return v;
}

void main()
{
    uint has_glow_effect = effect & uint(2);
    if (has_glow_effect > uint(0)) {
        // glowing effect
        const vec3 base_color = vec3(ivec3(247, 118, 34)) / 255.0;
        const vec3 half_base_color = base_color / 2.0;
        color = vec4(
            half_base_color * vec3(
                cos(- 10.0 * tex_coords.x) * sin(10.0 * tex_coords.y) * cos(t / 10.0),
                cos( 10.0 * tex_coords.x) * sin(10.0 * tex_coords.y),
                cos( 10.0 * tex_coords.x) * sin(- 10.0 * tex_coords.y)
            ) + half_base_color,
            1.0
        );
        return;
    }
    if (kind == uint(0)) {
        // texture
        color = true_tex_color(texture_rgba, tex_coords, float(layer));
    } else if (kind == uint(1)) {
        // text
        color = true_tex_color(texture_gray, tex_coords, float(layer));
    } else if (kind == uint(2)) {
        // solid colored shape
        color = vec4(effect_color, 1.0);
    }

    uint has_noise_effect = effect & uint(4);
    if (has_noise_effect > uint(0)) {
        vec3 noise = texture(texture_rgba, vec3(gl_FragCoord.x / 2048.0 + t / 10000.0, gl_FragCoord.y / 2048.0 + t / 5000.0, secondary_layer)).rgb;
        const vec3 gold = vec3(ivec3(255, 208, 0)) / 255.0;
        color = vec4(
            mix(effect_color.rgb, gold, vec3(noise.g, noise.b, (noise.g+noise.b) / 2.0)),
            color.a
        );
    }

    uint has_border_effect = effect & uint(8);
    if (has_border_effect > uint(0)) {
        float v = 0.0;
        if (kind == uint(1)) {
            v = get_border_alpha(texture_gray, tex_coords, float(layer), vec2(1.0, 1.0) / 2048.0);
        } else {
            v = get_border_alpha(texture_rgba, tex_coords, float(layer), vec2(1.0, 1.0) / 1024.0);
        }
        if (v > 0.0) {
            color = blend(color, vec4(0.0, 0.0, 0.0, v));
        }
    }
}