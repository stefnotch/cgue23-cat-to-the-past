#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(set = 0, binding = 0) uniform Scene {
    mat4 projView;
    vec3 lightPos;
    vec3 cameraPosition;
    float rewindTime;
} scene;

layout(set = 1, binding = 0) uniform Entity {
    mat4 model;
    mat4 normalMatrix;
} entity;

#include "../time_rewinding.glsl"

void main() {
    vec4 worldPos = entity.model * vec4(position, 1.0); // world space
    worldPos = vec4(timeRewindPosition(worldPos.xyz, scene.cameraPosition), worldPos.w);

    vec3 toLight = normalize(scene.lightPos - worldPos.xyz);

    vec3 n = mat3(entity.normalMatrix) * normal; // world space

    float angleFactor = 1.0 - max(dot(n, toLight), 0.0);

    float bias = max(0.09 * angleFactor, 0.0005);
    worldPos.xyz -= toLight * bias;

    vec4 clipSpacePosition = scene.projView * worldPos;
    gl_Position = clipSpacePosition;
}