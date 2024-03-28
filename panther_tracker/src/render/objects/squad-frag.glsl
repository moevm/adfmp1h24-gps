#version 320 es

precision mediump float;

vec3 v_color = vec3(1.0, 0.3, 0.1);
in vec2 v_position; // normalized position where x 0..1, y 0..v_y_ration
in float v_y_ratio;

out vec4 fragColor;

uniform vec3 circle; // x, y for circle center and z for its radius

void main() {
    float dist = distance(v_position, circle.xy);
    if (dist > circle.z) {
        fragColor = vec4(v_color, 0.0);
    } else {
        fragColor = vec4(v_color, 1.0);
    }
}
