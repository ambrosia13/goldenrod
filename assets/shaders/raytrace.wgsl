#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/space.wgsl
#include assets/shaders/lib/raytrace/intersect.wgsl

@group(0) @binding(0)
var<storage> screen: ScreenUniform;

struct SphereListUniform {
    num_spheres: u32,
    list: array<Sphere>,
}

struct PlaneListUniform {
    num_planes: u32,
    list: array<Plane>,
}

struct AabbListUniform {
    num_aabbs: u32,
    list: array<Aabb>,
}

@group(1) @binding(0)
var<storage> spheres: SphereListUniform;

@group(1) @binding(1)
var<storage> planes: PlaneListUniform;

@group(1) @binding(2)
var<storage> aabbs: AabbListUniform;

@group(2) @binding(0)
var color_texture: texture_storage_2d<rgba32float, write>;

@group(2) @binding(1)
var color_texture_copy: texture_storage_2d<rgba32float, read>;

fn raytrace(ray: Ray) -> Hit {
    var closest_hit: Hit;

    for (var i = 0u; i < spheres.num_spheres; i++) {
        let sphere = spheres.list[i];

        let hit = ray_sphere_intersect(ray, sphere);
        closest_hit = merge_hit(closest_hit, hit);
    }

    for (var i = 0u; i < planes.num_planes; i++) {
        let plane = planes.list[i];

        let hit = ray_plane_intersect(ray, plane);
        closest_hit = merge_hit(closest_hit, hit);
    }

    for (var i = 0u; i < aabbs.num_aabbs; i++) {
        let aabb = aabbs.list[i];

        let hit = ray_aabb_intersect(ray, aabb);
        closest_hit = merge_hit(closest_hit, hit);
    }

    return closest_hit;
}

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

    let hit = raytrace(ray);

    if hit.success {
        color = hit.material.albedo;
    }

    textureStore(color_texture, global_id.xy, vec4(pow(color, vec3(2.2)), 1.0));
}