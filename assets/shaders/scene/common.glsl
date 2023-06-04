struct PointLight {
    vec3 position;
    vec3 color;
    float range;
    float intensity;
};

#define MAX_NUM_TOTAL_LIGHTS 32

layout(set = 0, binding = 0) uniform Scene {
    PointLight pointLights[MAX_NUM_TOTAL_LIGHTS];
    int numLights;
    vec3 nearestShadowLight;
    float rewindTime;
} scene;

layout(set = 0, binding = 1) uniform samplerCubeShadow shadowMap;

layout(set = 1, binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
    vec3 position;
} camera;

layout(set = 2, binding = 0) uniform Material {
    vec3 baseColor;
    float roughness;
    float metallic;
    vec3 emissivity;
} material;

layout(set = 2, binding = 1) uniform sampler2D baseColorTexture;

layout(set = 3, binding = 0) uniform Entity {
    mat4 model;
    mat4 normalMatrix;
} entity;