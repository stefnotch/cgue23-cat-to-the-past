#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

const float PI = 3.14159265359;

#include "common.glsl"

vec3 ambientLightColor = vec3(1.0, 1.0, 1.0);

// n: normalized normal
// l: normalized vector pointing to the light source
// v: normalized view vector pointing to the camera
// h: normalized half-way vector between v and l

float vectorToDepthValue(vec3 direction) {
    vec3 absDirection = abs(direction);
    float localZ = max(absDirection.x, max(absDirection.y, absDirection.z));

    const float far = 100.0;
    const float near = 0.1;
    float normalizedZ =  (far) / (far - near) - (near*far)/(localZ * (far - near));
    return normalizedZ;
}

float computeShadowFactor(vec3 l) {
    float shadowDepth = texture(shadowMap, l).r;
    //return abs(vectorToDepthValue(l) - shadowDepth);
    const float bias = 0.015;
    if (shadowDepth + bias > vectorToDepthValue(l)) {
        return 1.0;
    } else {
        return 0.0;
    }
}

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

    return nDotx / max(nDotx * (1.0 - k) + k, 0.000001);
}


float geometrySmith(vec3 n, vec3 v, vec3 l, float alpha) {
    return geometrySchlickBeckmann(n, v, alpha) * geometrySchlickBeckmann(n, l, alpha);
}

vec3 fresnelSchlick(vec3 f0, vec3 v, vec3 h) {
    float vDoth = max(dot(v, h), 0.0);

    return f0 + (1.0 - f0) * pow(1.0 - vDoth, 5.0);
}

vec3 pbr_common(vec3 lightIntensity, vec3 l, vec3 n, vec3 v, vec3 albedo, vec3 f0) {
    vec3 h = normalize(v + l);

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
    kd *= 1.0-material.metallic;

    vec3 diffuseBRDF = kd * fLambert;
    vec3 specularBRDF = /* ks + */ fCookTorrance;
    float nDotL = max(dot(n, l), 0.0);

    return (diffuseBRDF + specularBRDF) * lightIntensity * nDotL;
}

vec3 pbr(PointLight pointLight, vec3 n, vec3 v, vec3 worldPos, vec3 albedo, vec3 f0) {
    vec3 positionToLight = pointLight.position - worldPos;
    vec3 l = normalize(positionToLight);
    float dSquared = max(dot(positionToLight, positionToLight), 0.000001);

    float attenuation = 1.0 / dSquared;
    vec3 lightIntensity = pointLight.color * pointLight.intensity * attenuation;
    return pbr_common(lightIntensity, l, n, v, albedo, f0);
}

void main() {
    vec3 worldPos = v_position;

    vec3 n = normalize(v_normal);
    vec3 v = normalize(camera.position - worldPos); // world space

    vec3 albedo = texture(baseColorTexture, v_uv).rgb * material.baseColor;

    // reflectance at normal incidence (base reflectance)
    // if dia-electric (like plastic) use F0 of 0.04 and if it's a metal, use the albedo as F0 (metallic workflow)
    vec3 f0 = vec3(0.04);
    f0 = mix(f0, albedo, material.metallic);

    // out going light
    vec3 Lo = vec3(0.0);

    for (int i = 0; i < scene.numLights; ++i) {
        Lo += pbr(scene.pointLights[i], n, v, worldPos, albedo, f0);
    }

    float ka = 0.03;
    vec3 ambient = (ambientLightColor * ka) * albedo;

    vec3 positionToNearestShadowLight = scene.nearestShadowLight - worldPos;
    vec3 l = positionToNearestShadowLight;

    vec3 color = Lo * computeShadowFactor(l)  + ambient;

    //f_color = vec4(color + material.emissivity, 1.0);

    f_color = vec4(vec3(1.0) * computeShadowFactor(l), 1.0);
}