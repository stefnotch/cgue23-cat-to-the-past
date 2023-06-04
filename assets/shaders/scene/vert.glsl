#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_position;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 v_uv;

#include "common.glsl"

void main() {
    vec4 worldPos = entity.model * vec4(position, 1.0); // world space
    vec3 n = mat3(entity.normalMatrix) * normal; // world space
    worldPos += vec4(vec3(0.1) * fract(scene.rewindTime), 0.0);

    vec4 clipSpacePosition = camera.proj * camera.view * worldPos;
    gl_Position = clipSpacePosition;

    v_position = worldPos.xyz;
    v_normal = n;
    v_uv = uv;
}