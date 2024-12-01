module bloom_merge;

import "../lib/header.slang";
import "../lib/bloom.slang";

[[vk_binding(0, 1)]]
Texture2D colorTexture;

[[vk_binding(1, 1)]]
SamplerState colorSampler;

[[vk_binding(2, 1)]]
Texture2D upsampleTexture;

[[vk_binding(3, 1)]]
SamplerState upsampleSampler;

[[vk_push_constant]]
LodInfo lodInfo;

[[shader("fragment")]]
float4 fragment(VertexOutput vertex): SV_Target {
    float4 color = colorTexture.Sample(colorSampler, vertex.uv);
    float4 bloom = upsampleTexture.Sample(upsampleSampler, vertex.uv);

    return max(float4(0.0), lerp(color, bloom, 0.2));
}