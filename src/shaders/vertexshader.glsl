#version 400
precision mediump float;

in vec4 a_position;

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

void main() {
    vec2 clipSpace = a_position.xy;
    gl_Position = vec4(clipSpace, 0.0, 1.0);
}