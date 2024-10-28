#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/space.wgsl
#include assets/shaders/lib/raytrace/intersect.wgsl

@group(0) @binding(0)
var<storage> screen: ScreenUniform;

struct SphereUniform {
    num_spheres: u32,
    list: array<Sphere>,
}

@group(1) @binding(0)
var<storage> spheres: SphereUniform;

@group(2) @binding(0)
var color_texture: texture_storage_2d<rgba32float, write>;

@group(2) @binding(1)
var color_texture_copy: texture_storage_2d<rgba32float, read>;

@compute
@workgroup_size(8, 8, 1)
fn compute(
    @builtin(local_invocation_id)
    local_id: vec3<u32>,
    @builtin(global_invocation_id)
    global_id: vec3<u32>,
) {
    if global_id.x >= screen.view.width || global_id.y >= screen.view.height {
        return;
    }

    var texcoord = vec2(f32(global_id.x), f32(global_id.y)) / vec2(f32(screen.view.width), f32(screen.view.height));
    texcoord.y = 1.0 - texcoord.y;

    let scaled_taa_offset = get_taa_offset(screen.view.frame_count) / vec2(f32(screen.view.width), f32(screen.view.height));
    let taa_offset_texcoord = texcoord + scaled_taa_offset;

    let screen_space_pos = vec3(taa_offset_texcoord, 1.0);
    let world_space_pos = from_screen_space(screen_space_pos, screen.camera.inverse_view_projection_matrix);
    let scene_space_pos = world_space_pos - screen.camera.position;

    let view_dir = normalize(scene_space_pos);

    var ray: Ray;
    ray.pos = screen.camera.position;
    ray.dir = view_dir;

    var color = vec3(max(vec3(0.0), ray.dir));

    var scene_hit: Hit;

    for (var i = 0u; i < spheres.num_spheres; i++) {
        let sphere = spheres.list[i];

        let hit = ray_sphere_intersect(ray, sphere);
        scene_hit = merge_hit(scene_hit, hit);
    }

    if scene_hit.success {
        color = scene_hit.material.albedo;
    }

    textureStore(color_texture, global_id.xy, vec4(pow(color, vec3(2.2)), 1.0));
}