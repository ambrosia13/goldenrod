module final;

import "lib/header.slang";

[[vk::binding(0, 1)]]
StructuredBuffer<ScreenUniform> screen;

[[vk::binding(0, 2)]]
Texture2D colorTexture;

[[vk::binding(1, 2)]]
SamplerState colorSampler;

static const float3x3 FRX_ACES_INPUT_MATRIX = float3x3(
    float3(0.59719, 0.07600, 0.02840),
    float3(0.35458, 0.90834, 0.13383),
    float3(0.04823, 0.01566, 0.83777)
);

// ODT_SAT => XYZ => D60_2_D65 => sRGB
static const float3x3 FRX_ACES_OUTPUT_MATRIX = float3x3(
    float3(1.60475, -0.10208, -0.00327),
    float3(-0.53108, 1.10813, -0.07276),
    float3(-0.07367, -0.00605, 1.07602)
);

float3 FRX_RRT_AND_ODTF_FIT(float3 v) {
    float3 a = v * (v + 0.0245786) - 0.000090537;
    float3 b = v * (0.983729 * v + 0.4329510) + 0.238081;

    return a / b;
}

float3 frx_toneMap(float3 color) {
    color = mul(FRX_ACES_INPUT_MATRIX, color);
    color = FRX_RRT_AND_ODTF_FIT(color);

    return mul(FRX_ACES_OUTPUT_MATRIX, color);
}

[[shader("fragment")]]
float4 fragment(VertexOutput vertex): SV_Target {
    float4 color = colorTexture.Sample(colorSampler, vertex.uv);

    color.rgb = pow(color.rgb, float3(1.0 / 2.2));
    color.rgb = frx_toneMap(color.rgb);

    return color;
}