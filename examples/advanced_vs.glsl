#version 330 core
#define DEG_TO_RAD 0.0174532925

layout (location = 0) in vec2 pos;
// (w, y, w, h)
layout (location = 1) in vec4 crop;
// layout2, because it's a mat4, also takes positions 3,4 and 5.
layout (location = 2) in vec2 translation;
layout (location = 3) in vec2 scale;
layout (location = 4) in vec2 rot_pivot;
layout (location = 5) in float rotation;
layout (location = 6) in uint _kind;
layout (location = 7) in uint _layer;
layout (location = 8) in uint _secondary_layer;
layout (location = 9) in uint _effect;
layout (location = 10) in vec3 _effect_color;

out vec2 tex_coords;
flat out uint kind;
flat out uint layer;
flat out uint secondary_layer;
flat out uint effect;
out vec3 effect_color;

uniform mat4 view;

void main()
{
    float rot = rotation * DEG_TO_RAD;
    mat4 model = mat4(
        1.0    , 0.0    , 0.0    , 0.0,
        0.0    , 1.0    , 0.0    , 0.0,
        0.0    , 0.0    , 1.0    , 0.0,
        translation.x , translation.y , 0.0, 1.0
    ) * mat4(
        1.0    , 0.0    , 0.0    , 0.0,
        0.0    , 1.0    , 0.0    , 0.0,
        0.0    , 0.0    , 1.0    , 0.0,
        rot_pivot.x, rot_pivot.y , 0.0, 1.0
    ) * mat4(
        cos(rot),-sin(rot), 0.0    , 0.0,
        sin(rot), cos(rot), 0.0    , 0.0,
        0.0     , 0.0     , 1.0    , 0.0,
        0.0     , 0.0     , 0.0    , 1.0
    ) * mat4(
        1.0    , 0.0    , 0.0    , 0.0,
        0.0    , 1.0    , 0.0    , 0.0,
        0.0    , 0.0    , 1.0    , 0.0,
        -rot_pivot.x, -rot_pivot.y, 0.0, 1.0
    )* mat4(
        scale.x, 0.0    , 0.0    , 0.0,
        0.0    , scale.y, 0.0    , 0.0,
        0.0    , 0.0    , 1.0    , 0.0,
        0.0    , 0.0    , 0.0    , 1.0
    );

    gl_Position = view * model * vec4(pos, 0.0, 1.0);

    tex_coords = vec2(
        crop.x + pos.x * crop.z,
        crop.y + pos.y * crop.w
    );
    kind = _kind;
    layer = _layer;
    secondary_layer = _secondary_layer;
    effect = _effect;
    effect_color = _effect_color;
}