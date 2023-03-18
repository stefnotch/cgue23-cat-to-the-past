#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_position;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 v_uv;

struct PointLight {
    vec3 position;
    vec3 color;
    float range;
    float intensity;
};

layout(set = 0, binding = 0) uniform Scene {
    PointLight pointLight;
} scene;

layout(set = 1, binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
    vec3 position;
} camera;

layout(set = 2, binding = 0) uniform Material {
    vec3 base_color;
    float roughness;
    float metallic;
    vec3 emissivity;
} material;

layout(set = 2, binding = 1) uniform sampler2D base_color_texture;

layout(set = 3, binding = 0) uniform Entity {
    mat4 model;
    mat4 normalMatrix;
} entity;

void main() {
    vec3 worldPos = (entity.model * vec4(position, 1.0)).xyz; // world space
    vec3 n = mat3(entity.normalMatrix) * normal; // world space

    gl_Position = camera.proj * camera.view * entity.model * vec4(position, 1.0);

    v_position = worldPos;
    v_normal = n;
    v_uv = uv;
}