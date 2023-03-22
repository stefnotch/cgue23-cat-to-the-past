#version 450

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D image;

void main() {
    f_color = texture(image, v_uv);
//    f_color = vec4(1.0);
}