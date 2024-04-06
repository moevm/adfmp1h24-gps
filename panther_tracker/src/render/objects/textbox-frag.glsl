#version 320 es
precision highp float;

uniform sampler2D tex;

in vec2 v_position;
in vec2 v_texcoord;

out vec4 fragColor;

void main() {

    fragColor = vec4(1.0, 0.0, 0.0, texture(tex, v_texcoord).r);
}