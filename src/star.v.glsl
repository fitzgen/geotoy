#version 140

in float x;
in float y;

in vec2 attractor;
in uint kind;

uniform float a;
uniform float b;

void main() {
    float multiplier = kind == uint(2) ? 0 : (kind == uint(0) ? b : a);

    vec2 p = vec2(x, y);
    vec2 v = attractor - p;
    gl_Position = vec4(p + multiplier * v, 0.0, 1.0);
}
