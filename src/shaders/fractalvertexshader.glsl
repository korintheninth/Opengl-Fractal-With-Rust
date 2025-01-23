#version 300 es
precision mediump float;

uniform vec4 u_position;

void main() {
    vec2 clipSpace = a_position.xy;
    gl_Position = vec4(clipSpace, 0.0, 1.0);
}