#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;


layout(push_constant) uniform PushConsts {
    mat4 projView;
    mat4 model;
} push;

layout(set = 0, binding = 0) uniform Scene {
    float rewindTime;
} scene;

// lightPos
// normalMatrix
#include "../time_rewinding.glsl"


void main() {
    vec4 worldPos = push.model * vec4(position, 1.0); // world space
    worldPos = vec4(timeRewindPosition(worldPos.xyz), worldPos.w);

    vec4 clipSpacePosition = push.projView * worldPos;
    gl_Position = clipSpacePosition;
}