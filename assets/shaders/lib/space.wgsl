fn from_screen_space(screen_space_pos: vec3<f32>, matrix: mat4x4<f32>) -> vec3<f32> {
    let clip_space_pos = screen_space_pos * 2.0 - 1.0;
    let temp = matrix * vec4(clip_space_pos, 1.0);
    return temp.xyz / temp.w;
}

fn to_screen_space(pos: vec3<f32>, matrix: mat4x4<f32>) -> vec3<f32> {
    let temp = matrix * vec4(pos, 1.0);
    return (temp.xyz / temp.w) * 0.5 + 0.5;
}