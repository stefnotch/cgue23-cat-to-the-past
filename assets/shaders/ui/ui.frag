#version 450

layout(location = 0) out vec4 f_color;

layout(location = 1) in vec2 v_uv;

layout(set = 0, binding = 0) uniform sampler2D image;

void main() {
    vec4 image_color = texture(image, v_uv);
    if (image_color.a < 0.5) {
        discard;
    }

    f_color = vec4(image_color.rgb, 1.0);
}