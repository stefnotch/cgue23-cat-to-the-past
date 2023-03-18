#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

const float PI = 3.14159265359;

// TODO: import structs to reduce code duplication and to keep the structs in sync in both vertex and fragment shader

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
    // TODO: change name to camelCase
    vec3 base_color;
    float roughness;
    float metallic;
    vec3 emissivity;
} material;

// TODO: change name to camelCase
layout(set = 2, binding = 1) uniform sampler2D base_color_texture;

layout(set = 3, binding = 0) uniform Entity {
    mat4 model;
    mat4 normalMatrix;
} entity;

vec3 ambientLightColor = vec3(1.0, 1.0, 1.0);

// n: normalized normal
// l: normalized vector pointing to the light source
// v: normalized view vector pointing to the camera
// h: normalized half-way vector between v and l

float distributionGGXTrowbridgeReitz(vec3 n, vec3 h, float alpha) {
    float alphaSquared = alpha * alpha;

    float nDoth = max(dot(n,h), 0.0);
    float nDothSquared = nDoth * nDoth;

    float partDenom = nDothSquared * (alphaSquared - 1.0) + 1.0;

    return alphaSquared / (PI * partDenom * partDenom);
}

// x: in this context only v or l are allowed to be x
float geometrySchlickBeckmann(vec3 n, vec3 x, float alpha) {
    float k = alpha / 2.0; // there are other options for this
    float nDotx = max(dot(n, x), 0.0);

    return nDotx / (nDotx * (1.0 - k) + k);
}


float geometrySmith(vec3 n, vec3 v, vec3 l, float alpha) {
    return geometrySchlickBeckmann(n, v, alpha) * geometrySchlickBeckmann(n, l, alpha);
}

vec3 fresnelSchlick(vec3 f0, vec3 v, vec3 h) {
    float vDoth = max(dot(v, h), 0.0);

    return f0 + (1.0 - f0) * pow(1.0 - vDoth, 5.0);
}

vec3 pbr(PointLight pointLight, vec3 n, vec3 v, vec3 worldPos, vec3 albedo, vec3 f0) {
    vec3 positionToLight = pointLight.position - worldPos;
    vec3 l = normalize(positionToLight);
    vec3 h = normalize(v + l);
    float dSquared = dot(positionToLight, positionToLight);

    float attenuation = 1.0 / dSquared;

    vec3 fLambert = albedo / PI;

    float alpha = material.roughness * material.roughness;

    // D: Normal Distribution Function (GGX/Trowbridge-Reitz)
    float D = distributionGGXTrowbridgeReitz(n, h, alpha);

    // G: Geometry Function (Smith Model using Schlick-Beckmann)
    float G = geometrySmith(n, v, l, alpha);

    // F: Fresnel Function
    vec3 F = fresnelSchlick(f0, v, h);

    vec3 fCookTorranceNumerator = D * G * F;
    float fCookTorranceDenominator = 4.0 * max(dot(n, l), 0.0) * max(dot(n, v), 0.0);
    fCookTorranceDenominator = max(fCookTorranceDenominator, 0.000001);

    vec3 fCookTorrance =  fCookTorranceNumerator / fCookTorranceDenominator;

    vec3 ks = F;
    vec3 kd = vec3(1.0) - ks;

    vec3 diffuseBRDF = kd * fLambert;
    vec3 specularBRDF = /* ks + */ fCookTorrance;
    vec3 lightIntensity = pointLight.color * pointLight.intensity * attenuation;
    float nDotL = max(dot(n, l), 0.0);

    return (diffuseBRDF + specularBRDF) * lightIntensity * nDotL;
}

void main() {
    vec3 worldPos = v_position;

    vec3 n = normalize(v_normal);
    vec3 v = normalize(camera.position - worldPos); // world space

    vec3 albedo = texture(base_color_texture, v_uv).rgb * material.base_color;

    // reflectance at normal incidence (base reflectance)
    // if dia-electric (like plastic) use F0 of 0.04 and if it's a metal, use the albedo as F0 (metallic workflow)
    vec3 f0 = vec3(0.04);
    f0 = mix(f0, albedo, material.metallic);

    // out going light
    vec3 Lo = vec3(0.0);

    // we only have one light for now
    Lo += pbr(scene.pointLight, n, v, worldPos, albedo, f0);

    float ka = 0.03;
    vec3 ambient = (ambientLightColor * ka) * albedo;

    f_color = vec4(ambient + Lo, 1.0);
}