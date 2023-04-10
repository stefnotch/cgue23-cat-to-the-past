#version 450

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D image;

float exposure = 2.0;

// Source: https://github.com/Shot511/RapidGL/blob/65d1202a5926acad9816483b141fb24480e81668/src/demos/22_pbr/tmo.frag
// sRGB => XYZ => D65_2_D60 => AP1 => RRT_SAT
mat3 ACESInputMatrix = {
    {0.59719, 0.07600, 0.02840},
    {0.35458, 0.90834, 0.13383},
    {0.04823, 0.01566, 0.83777}
};

// ODT_SAT => XYZ => D60_2_D65 => sRGB
mat3 ACESOutputMatrix = {
    { 1.60475, -0.10208, -0.00327},
    {-0.53108,  1.10813, -0.07276},
    {-0.07367, -0.00605,  1.07602 }
};

vec3 RRTAndODTFit(vec3 v) {
    vec3 a = v * (v + 0.0245786f) - 0.000090537f;
    vec3 b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
    return a / b;
}

void main() {
    vec3 color = exposure * texture(image, v_uv).rgb;

    color = ACESInputMatrix * color.rgb;
    color = RRTAndODTFit(color);
    color = ACESOutputMatrix * color;

    f_color = vec4(color, 1.0);
}