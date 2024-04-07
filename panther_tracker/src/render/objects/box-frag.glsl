#version 300 es
precision highp float;

uniform vec3 color;
uniform float y_ratio;

in vec2 v_position; // normalized position where x 0..1, y 0..y_ratio
in vec2 v_texcoord;

out vec4 fragColor;

void main() {
    fragColor = vec4(color, 1.0);
}
