module bloom;

import "header.slang";

public struct LodInfo {
    public uint currentLod;
    public uint maxLod;
}

// Bloom sampling filters taken from FREX, GPL license
// https://github.com/vram-guild/frex/blob/1.19/common/src/main/resources/assets/frex/shaders/lib/sample.glsl

// Used for bloom downsample
public float4 frx_sample13(
    Texture2D texture,
    SamplerState sampler,
    float2 uv,
    uint2 resolution
) {
    float2 dist = 1.0 / float2(resolution);

    float4 a = texture.Sample(sampler, uv + dist * float2(-1.0, -1.0));
    float4 b = texture.Sample(sampler, uv + dist * float2( 0.0, -1.0));
    float4 c = texture.Sample(sampler, uv + dist * float2( 1.0, -1.0));
    float4 d = texture.Sample(sampler, uv + dist * float2(-0.5, -0.5));
    float4 e = texture.Sample(sampler, uv + dist * float2( 0.5, -0.5));
    float4 f = texture.Sample(sampler, uv + dist * float2(-1.0,  0.0));
    float4 g = texture.Sample(sampler, uv + dist);
    float4 h = texture.Sample(sampler, uv + dist * float2( 1.0,  0.0));
    float4 i = texture.Sample(sampler, uv + dist * float2(-0.5,  0.5));
    float4 j = texture.Sample(sampler, uv + dist * float2( 0.5,  0.5));
    float4 k = texture.Sample(sampler, uv + dist * float2(-1.0,  1.0));
    float4 l = texture.Sample(sampler, uv + dist * float2( 0.0,  1.0));
    float4 m = texture.Sample(sampler, uv + dist * float2( 1.0,  1.0));

    float2 div = (1.0 / 4.0) * float2(0.5, 0.125);

    return 
        (d + e + i + j) * div.x +
        (a + b + g + f) * div.y +
        (b + c + h + g) * div.y +
        (f + g + l + k) * div.y +
        (g + h + m + l) * div.y;

}

// Used for bloom upsample
public float4 frx_sampleTent(
    Texture2D texture,
    SamplerState sampler,
    float2 uv,
    uint2 resolution
) {
    float2 dist = 1.0 / float2(resolution);
    float4 d = float4(1.0, 1.0, -1.0, 0.0) * dist.xyxy;

    float4 sum = float4(0.0);

    sum += texture.Sample(sampler, uv - d.xy);
    sum += texture.Sample(sampler, uv - d.wy) * 2.0;
    sum += texture.Sample(sampler, uv - d.zy);
    sum += texture.Sample(sampler, uv + d.zw) * 2.0;
    sum += texture.Sample(sampler, uv) * 4.0;
    sum += texture.Sample(sampler, uv + d.xw) * 2.0;
    sum += texture.Sample(sampler, uv + d.zy);
    sum += texture.Sample(sampler, uv + d.wy) * 2.0;
    sum += texture.Sample(sampler, uv + d.xy);

    return sum * (1.0 / 16.0);
}

// Exponential weight for bloom LODs
public float getSampleWeight(uint currentLod, uint maxLod) {
    float x = float(currentLod);
    float n = float(maxLod);

    return exp(-x / n) / ((1.0 - 1.0 / E) * n);
}