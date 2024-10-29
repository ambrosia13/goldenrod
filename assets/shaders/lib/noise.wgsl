var<private> rng_state: u32;
var<private> static_rng_state: u32;

fn init_rng(frag_coord: vec2<u32>, view_width: u32, view_height: u32, frame_count: u32) {
    let rng_ptr = &rng_state;
    let static_rng_ptr = &static_rng_state;
    *rng_ptr = u32(view_width * view_height) * (frame_count + 1) * u32(frag_coord.x + frag_coord.y * view_width);
    *static_rng_ptr = frame_count + 1u;
}

fn pcg(seed: ptr<private, u32>) {
    let state: u32 = *seed * 747796405u + 2891336453u;
    let word: u32 = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    *seed = (word >> 22u) ^ word;
}

fn next_u32() -> u32 {
    pcg(&rng_state);
    return rng_state;
}

fn next_f32() -> f32 {
    return f32(next_u32()) / f32(0xFFFFFFFFu);
}

fn generate_unit_vector() -> vec3<f32> {
    var xy = vec2(next_f32(), next_f32());
    xy.x *= TAU;
    xy.y = 2.0 * xy.y - 1.0;

    return (vec3(vec2(sin(xy.x), cos(xy.x)) * sqrt(1.0 - xy.y * xy.y), xy.y));
}

fn next_u32_static() -> u32 {
    pcg(&static_rng_state);
    return static_rng_state;
}

fn next_f32_static() -> f32 {
    return f32(next_u32_static()) / f32(0xFFFFFFFFu);
}

fn generate_unit_vector_static() -> vec3<f32> {
    var xy = vec2(next_f32_static(), next_f32_static());
    xy.x *= TAU;
    xy.y = 2.0 * xy.y - 1.0;

    return (vec3(vec2(sin(xy.x), cos(xy.x)) * sqrt(1.0 - xy.y * xy.y), xy.y));
}

fn generate_cosine_vector(normal: vec3<f32>) -> vec3<f32> {
    return normalize(normal + generate_unit_vector());
}

fn generate_cosine_vector_from_roughness(normal: vec3<f32>, roughness: f32) -> vec3<f32> {
    return normalize(normal + generate_unit_vector() * roughness);
}
