#version 450

layout(location = 0) in vec3 v_normal;

layout(location = 0) out vec4 f_color;

const vec3 light_dir = vec3(0.2f, -1.0f, 0.3f);
const vec3 light_color = vec3(1.0f);

const vec3 model_color = vec3(0.2f, 0.4f, 0.3f);

void main() {
    vec3 l = -normalize(light_dir);
    vec3 n = v_normal;

    vec3 diffuse = max(dot(n, l), 0) * light_color;

    f_color = vec4(diffuse * model_color, 1.0);
}