#version 450

layout(location = 0) out vec4 f_color;

layout(location = 1) in vec2 v_uv;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput input_image;

layout(set = 1, binding = 0) uniform sampler2D image;

void main() {
    vec4 input_color = subpassLoad(input_image);
    vec4 image_color = texture(image, v_uv);

    vec3 mixedColor = mix(input_color.rgb, image_color.rgb, image_color.a);

    f_color = vec4(mixedColor, 1.0);
}