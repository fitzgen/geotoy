#version 140

in float x;
in float y;

in vec2 attractor;
in uint kind;

uniform float a;
uniform float b;
uniform vec2 offset;

void main() {
    float multiplier = kind == uint(2) ? 0 : (kind == uint(1) ? a : b);

    vec2 p = vec2(x, y) + offset;
    vec2 v = attractor + offset - p;
    gl_Position = vec4(p + multiplier * v, 0.0, 1.0);
}
