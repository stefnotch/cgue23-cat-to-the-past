#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_normal;

layout(location = 0) out vec4 f_color;

struct Attenuation {
    float constant;
    float linear;
    float quadratic;
};

struct Material {
    vec3 color;
    float ka;
    float kd;
    float ks;
    float alpha;
};

struct PointLight {
    vec3 position;
    vec3 color;
    Attenuation attenuation;
};

layout(set = 0, binding = 0) uniform Scene {
    PointLight pointLight;
} scene;

layout(set = 1, binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
    vec3 position;
} camera;

layout(set = 2, binding = 0) uniform Entity {
    mat4 model;
    mat4 normalMatrix;
    Material material;
} entity;

vec3 ambientLightColor = vec3(1.0, 1.0, 1.0);

// ka: ambient reflection constant
// ia: ambient intensity
vec3 ambient(float ka, vec3 ia) {
    return ka * ia;
}

// kd: diffuse reflection constant
// id: diffuse intensity
// n: normalized normal
// l: normalized vector pointing to the light source
vec3 diffuse(float kd, vec3 n, vec3 l, vec3 id) {
    return kd * max(dot(n, l), 0) * id;
}

// ks: specular reflection constant
// alpha: shininess constant
// r: reflected light direction
// v: normalized view vector pointing to the camera
vec3 specular(float ks, float alpha, vec3 r, vec3 v, vec3 is) {
    return ks * pow(max(dot(v, r), 0), alpha) * is;
}

vec3 phong(Material material, PointLight pointLight, vec3 n, vec3 v, vec3 worldPos) {
    vec3 l = normalize(pointLight.position - worldPos);
    float d = length(pointLight.position - worldPos);
    vec3 r = reflect(-l,n);

    Attenuation att = pointLight.attenuation;
    float reciAttenuation = 1.0 / (att.constant + d * att.linear + d * d * att.quadratic);

    return (diffuse(material.kd, n, l, pointLight.color * material.color) +
    specular(material.ks, material.alpha, r, v, pointLight.color)) * reciAttenuation;
}

void main() {
    vec3 worldPos = v_position;

    vec3 n = normalize(v_normal);
    vec3 v = normalize(camera.position - worldPos); // world space

    f_color = vec4(ambient(entity.material.ka, ambientLightColor * entity.material.color) +
        phong(entity.material, scene.pointLight, n, v, worldPos)
    , 1.0);
}