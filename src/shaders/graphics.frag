#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout (location = 0) in vec2 fragTexCoord;

layout (location = 0) out vec4 outColor;

layout (binding = 1) uniform sampler texSampler;
layout (binding = 2) uniform texture2D tex;

void main() {
    outColor = texture(sampler2D(tex, texSampler), fragTexCoord);
}
