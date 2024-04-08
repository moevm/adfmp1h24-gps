#version 300 es
precision highp float;

uniform float y_ratio;
uniform float t;

in vec2 v_position; // normalized position where x 0..1, y 0..y_ratio
in vec2 v_texcoord;

out vec4 fragColor;

vec4 fillColor(float t, vec4 color) {
    t = sin(t * 3.0) * 0.5 + 0.5;

    float side_offs = 0.45;
    side_offs -= 0.15 * t;

    float top_offs = 0.2;
    top_offs -= 0.2 * t;

    float initial_sickness = 0.02;
    float bot_side = 1.0 + initial_sickness;
    float top_side = bot_side + top_offs;

    bool colored = false;

    vec2 pt = vec2(side_offs, bot_side);
    if (distance(pt, v_position) < initial_sickness) {
        colored = true;
    }

    pt = vec2(1.0 - side_offs, bot_side);
    if (distance(pt, v_position) < initial_sickness) {
        colored = true;
    }

    pt = vec2(side_offs, top_side);
    if (distance(pt, v_position) < initial_sickness) {
        colored = true;
    }

    pt = vec2(1.0 - side_offs, top_side);
    if (distance(pt, v_position) < initial_sickness) {
        colored = true;
    }

    //rect1
    if (v_position.x > side_offs - initial_sickness && v_position.x < 1.0 - side_offs + initial_sickness
     && v_position.y > bot_side && v_position.y < top_side) {
        colored = true;
    }

    //rect2
    if (v_position.y > bot_side - initial_sickness && v_position.y < top_side + initial_sickness
     && v_position.x > side_offs && v_position.x < 1.0 - side_offs) {
        colored = true;
    }

    if (colored) {
        return color;
    } else {
        return vec4(0.0); // return transparent color if not colored
    }
}

void main() {
    vec4 color = vec4(1.0, 0.85, 1.0, 1.0);
    fragColor = fillColor(t + 0.1, color);
    if (fragColor.a > 0.0) {
        return;
    }

    color = vec4(0.5, 0.2, 0.9, 1.0);
    fragColor = fillColor(t + 0.066, color);
    if (fragColor.a > 0.0) {
        return;
    }

    color = vec4(0.6, 0.8, 0.2, 1.0);
    fragColor = fillColor(t + 0.033, color);
    if (fragColor.a > 0.0) {
        return;
    }

    color = vec4(0.4, 0.5, 0.9, 1.0);
    fragColor = fillColor(t, color);


}
