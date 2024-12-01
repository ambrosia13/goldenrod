struct LodInfo {
    current_lod: u32,
    max_lod: u32,
}

// Bloom sampling filters taken from FREX
// https://github.com/vram-guild/frex/blob/1.19/common/src/main/resources/assets/frex/shaders/lib/sample.glsl

// used for bloom downsample
fn sample_13(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, resolution: vec2<u32>) -> vec4<f32> {
    let dist = 1.0 / vec2<f32>(resolution);

    let a = textureSample(tex, samp, uv + dist * vec2(-1.0, -1.0));
    let b = textureSample(tex, samp, uv + dist * vec2( 0.0, -1.0));
    let c = textureSample(tex, samp, uv + dist * vec2( 1.0, -1.0));
    let d = textureSample(tex, samp, uv + dist * vec2(-0.5, -0.5));
    let e = textureSample(tex, samp, uv + dist * vec2( 0.5, -0.5));
    let f = textureSample(tex, samp, uv + dist * vec2(-1.0,  0.0));
    let g = textureSample(tex, samp, uv);
    let h = textureSample(tex, samp, uv + dist * vec2( 1.0,  0.0));
    let i = textureSample(tex, samp, uv + dist * vec2(-0.5,  0.5));
    let j = textureSample(tex, samp, uv + dist * vec2( 0.5,  0.5));
    let k = textureSample(tex, samp, uv + dist * vec2(-1.0,  1.0));
    let l = textureSample(tex, samp, uv + dist * vec2( 0.0,  1.0));
    let m = textureSample(tex, samp, uv + dist * vec2( 1.0,  1.0));

    let div = (1.0 / 4.0) * vec2(0.5, 0.125);

    return 
        (d + e + i + j) * div.x +
        (a + b + g + f) * div.y +
        (b + c + h + g) * div.y +
        (f + g + l + k) * div.y +
        (g + h + m + l) * div.y;
}

// used for bloom upsample
fn sample_tent(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, resolution: vec2<u32>) -> vec4<f32> {
    let dist = 1.0 / vec2<f32>(resolution);
    let d = vec4(1.0, 1.0, -1.0, 0.0) * dist.xyxy;

    var sum = vec4(0.0);

    sum += textureSample(tex, samp, uv - d.xy);
    sum += textureSample(tex, samp, uv - d.wy) * 2.0;
    sum += textureSample(tex, samp, uv - d.zy);
    sum += textureSample(tex, samp, uv + d.zw) * 2.0;
    sum += textureSample(tex, samp, uv) * 4.0;
    sum += textureSample(tex, samp, uv + d.xw) * 2.0;
    sum += textureSample(tex, samp, uv + d.zy);
    sum += textureSample(tex, samp, uv + d.wy) * 2.0;
    sum += textureSample(tex, samp, uv + d.xy);

    return sum * (1.0 / 16.0);
}

fn sample_weight(current_lod: u32, max_lod: u32) -> f32 {
    // constant weight
    // return 1.0 / f32(max_lod);

    // exponential weight
    let x = f32(current_lod);
    let n = f32(max_lod);

    return exp(-x / n) / ((1.0 - 1.0 / E) * n);

    // linear weight
    // let x = f32(current_lod);
    // let n = f32(max_lod);
    // let m = (n - 2.0) / 2.0 + 4.0;

    // return max(0.0, m - x) / n;

    // if current_lod <= 5 {
    //     return 1.0 / 5.0;
    // } else {
    //     return 0.0;
    // }
}
