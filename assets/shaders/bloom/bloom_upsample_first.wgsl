#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/bloom.wgsl

@group(1) @binding(0)
var downsample_texture: texture_2d<f32>;

@group(1) @binding(1)
var downsample_sampler: sampler;

var<push_constant> lod_info: LodInfo;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let weight = sample_weight(lod_info.current_lod, lod_info.max_lod);
    return textureSample(downsample_texture, downsample_sampler, in.uv) * weight;
}