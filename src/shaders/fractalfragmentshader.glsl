#version 300 es
precision highp float;

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;
uniform float u_scroll;

out vec4 fragColor;

float map(float value, float min1, float max1, float min2, float max2) {
    return min2 + (value - min1) * (max2 - min2) / (max1 - min1);
}

void main() {
    float scale = 1000.0;
    float a = map(gl_FragCoord.x, 0.0, u_resolution.x, -2.0 * (u_resolution.x / scale), 2.0 * (u_resolution.x / scale)) * u_scroll  + u_mouse.x;
    float b = map(gl_FragCoord.y, 0.0, u_resolution.y, -2.0 * (u_resolution.y / scale), 2.0 * (u_resolution.y / scale)) * u_scroll  + u_mouse.y;
    float ca = a;
    float cb = b;
    float aa = a * a;
    float bb = b * b;
    float ab2 = a * b * 2.0;
    int i;

    const int maxIterations = 5000;

    for (i = 0; i < maxIterations; i++) {
        if (aa + bb > 4.0) break;
        b = ab2 + cb;
        a = aa - bb + ca;
        aa = a * a;
        bb = b * b;
        ab2 = a * b * 2.0;
    }

    if (i == maxIterations) {
        fragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    
    float fi = float(i);
    vec3 color = vec3(
        abs(sin(fi * 0.1 * u_time)), 
        abs(cos(fi * 0.1 * u_time)), 
        abs(sin(fi * 0.1 * u_time) - cos(fi * 0.1 * u_time))
    );
    fragColor = vec4(color, 1.0);
}