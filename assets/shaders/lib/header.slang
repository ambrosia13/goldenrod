module header;

public static const float PI = 3.1415926535897932384626433832795;
public static const float HALF_PI = 1.57079632679489661923;
public static const float TAU = 6.2831853071795864769252867665590;

public static const float E = 2.718281828459045235360287471352;

public struct VertexOutput {
    public float4 clipPosition: SV_Position;

    [[vk_location(0)]]
    public float2 uv;
    
    [[vk_location(1)]]
    public float2 texcoord;
}

public struct Camera {
    public float4x4 viewProjectionMatrix;
    public float4x4 viewMatrix;
    public float4x4 projectionMatrix;

    public float4x4 inverseViewProjectionMatrix;
    public float4x4 inverseViewMatrix;
    public float4x4 inverseProjectionMatrix;

    public float4x4 previousViewProjectionMatrix;
    public float4x4 previousViewMatrix;
    public float4x4 previousProjectionMatrix;

    public float3 position;
    public float3 previousPosition;

    public float3 view;
    public float3 previousView;

    public float3 right;
    public float3 up;
}

public struct View {
    public uint width;
    public uint height;

    public uint frameCount;
}

public struct ScreenUniform {
    public Camera camera;
    public View view;
}

public float2 getTaaOffset(uint frame) {
    float2 taaOffsets[] = {
        float2( 0.125, -0.375),
        float2(-0.125,  0.375),
        float2( 0.625,  0.125),
        float2( 0.375, -0.625),
        float2(-0.625,  0.625),
        float2(-0.875, -0.125),
        float2( 0.375, -0.875),
        float2( 0.875,  0.875)
    };

    return taaOffsets[frame % 8];
}
