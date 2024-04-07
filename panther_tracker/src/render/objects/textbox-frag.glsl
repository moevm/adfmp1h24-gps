#version 320 es
precision highp float;

uniform sampler2D tex;

in vec2 v_position;
in vec2 v_texcoord;

out vec4 fragColor;

void main() {

    float intencity = texture(tex, v_texcoord).r;
    if (intencity > 0.01) {
        fragColor = vec4(1.0, 0.0, 0.0, intencity);
    }
    else {
        fragColor = vec4(0.5, 0.8, 0.9, 0.4);
    }
}