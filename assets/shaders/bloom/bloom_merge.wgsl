#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/bloom.wgsl

@group(1) @binding(0)
var color_texture: texture_2d<f32>;

@group(1) @binding(1)
var color_sampler: sampler;

@group(1) @binding(2)
var upsample_texture: texture_2d<f32>;

@group(1) @binding(3)
var upsample_sampler: sampler;

var<push_constant> lod_info: LodInfo;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(color_texture, color_sampler, in.uv);
    let bloom = textureSample(upsample_texture, upsample_sampler, in.uv);

    return max(vec4(0.0), mix(color, bloom, 0.2));
}