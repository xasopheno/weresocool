#version 450

layout(binding = 0) uniform texture2D tex;
layout(binding = 1) uniform sampler samp;

layout(location = 0) out vec4 outColor;

void main() {
    vec2 texCoord = gl_FragCoord.xy / vec2(2.0) + vec2(0.5);
    vec4 color = texture(sampler2D(tex, samp), texCoord);
    outColor = vec4(color.rgb, 1.0);
}
