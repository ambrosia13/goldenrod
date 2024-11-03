#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/bloom.wgsl

@group(1) @binding(0)
var previous_upsample_mip_texture: texture_2d<f32>;

@group(1) @binding(1)
var upsample_sampler: sampler;

@group(1) @binding(2)
var downsample_texture: texture_2d<f32>;

@group(1) @binding(3)
var downsample_sampler: sampler;

@group(1) @binding(4)
var<storage> screen: ScreenUniform;

var<push_constant> lod_info: LodInfo;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let prior = sample_tent(
        previous_upsample_mip_texture, 
        upsample_sampler, 
        in.uv, 
        vec2(screen.view.width >> lod_info.current_lod, screen.view.height >> lod_info.current_lod)
    );

    let weight = sample_weight(lod_info.current_lod, lod_info.max_lod);
    let current = textureSample(downsample_texture, downsample_sampler, in.uv) * weight;

    return prior + current;
}