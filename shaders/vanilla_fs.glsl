#version 330 core
#define M_PI 3.1415926535897932384626433832795
in vec2 TexCoords;

uniform float outline_width_x;
uniform float outline_width_y;
uniform vec3 outline_color;
uniform vec4 model_color_filter;
uniform vec4 model_color_blend;

out vec4 color;

uniform sampler2D image;

void main()
{
    vec4 texColor = texture(image, TexCoords);
    vec4 modelColor = vec4(
        mix((model_color_filter * texColor).rgb, model_color_blend.rgb, model_color_blend.a),
        texColor.a
    );
    if (outline_width_x > 0.0) {
        float max_alpha = 0.0;
        int total_checks = 9;
        for (int i = 0; i < total_checks; i++) {
            if (max_alpha >= 1.0) {
                break;
            }
            float angle = float(i) / float(total_checks) * 2.0 * M_PI;
            max_alpha = max(texture(image, vec2(TexCoords.x - sin(angle) * outline_width_x, TexCoords.y - cos(angle) * outline_width_y)).a, max_alpha); 
        } 
        color = mix(vec4(outline_color, max_alpha), modelColor, texColor.a);
    } else {
        color = modelColor;
    }
} 