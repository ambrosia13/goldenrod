@vertex
fn vertex() -> @builtin(position) vec4<f32> {
    return vec4(1.0);
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 0.0, 1.0);
}

@compute
@workgroup_size(1, 1, 1)
fn compute() {}