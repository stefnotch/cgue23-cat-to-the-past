#version 450

layout(location = 0) in vec3 position;

layout(push_constant) uniform PushConsts {
    mat4 projView;
    mat4 model;
} push;

void main() {
    vec4 clipSpacePosition = push.projView * push.model * vec4(position, 1.0);
    gl_Position = clipSpacePosition;
}