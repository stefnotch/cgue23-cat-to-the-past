#version 450

layout(location = 0) in vec3 position;


layout(push_constant) uniform PushConsts {
    mat4 projView;
    mat4 model;
} push;

layout(set = 0, binding = 0) uniform Scene {
    float rewindTime;
} scene;

// lightPos
// normalMatrix


void main() {
    vec4 worldPos = push.model * vec4(position, 1.0); // world space
    worldPos += vec4(vec3(0.1) * fract(scene.rewindTime), 0.0);

    vec4 clipSpacePosition = push.projView * worldPos;
    gl_Position = clipSpacePosition;
}