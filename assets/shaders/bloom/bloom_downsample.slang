module bloom_downsample;

import "../lib/header.slang";
import "../lib/bloom.slang";

[[vk_binding(0, 1)]]
Texture2D bloomTexture;

[[vk_binding(1, 1)]]
SamplerState bloomSampler;

[[vk_binding(2, 1)]]
GLSLShaderStorageBuffer<ScreenUniform> screen;

[[vk_push_constant]]
LodInfo lodInfo;

[[shader("fragment")]]
float4 fragment(VertexOutput vertex): SV_Target {
    float4 color = frx_sample13(
        bloomTexture,
        bloomSampler,
        vertex.uv,
        uint2(
            screen.view.width >> lodInfo.currentLod,
            screen.view.height >> lodInfo.currentLod,
        )
    );

    return color;
}