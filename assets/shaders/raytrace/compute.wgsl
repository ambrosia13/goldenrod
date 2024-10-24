// #include assets/shaders/lib/rt/intersect.wgsl

@group(0) @binding(0)
var color_texture: texture_storage_2d<rgba32float, write>;

@group(0) @binding(1)
var color_texture_copy: texture_storage_2d<rgba32float, read>;

@group(1) @binding(0)
var<storage> screen: ScreenUniforms;

@group(2) @binding(0)
var<storage> objects: ObjectsUniform;

@compute
@workgroup_size(16, 16, 1)
fn compute(
    @builtin(local_invocation_id)
    local_id: vec3<u32>,
    @builtin(global_invocation_id)
    global_id: vec3<u32>,
) {
    textureStore(color_texture, global_id.xy, vec4(f32(local_id.x) / 16.0, f32(local_id.y) / 16.0, 0.0, 1.0));
}