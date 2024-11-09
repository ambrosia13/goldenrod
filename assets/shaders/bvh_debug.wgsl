#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/space.wgsl
#include assets/shaders/lib/noise.wgsl
#include assets/shaders/lib/raytrace/stack.wgsl
#include assets/shaders/lib/raytrace/intersect.wgsl
#include assets/shaders/lib/raytrace/bvh.wgsl

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

struct BvhUniform {
    num_nodes: u32,
    nodes: array<BvhNode>,
}

// struct BvhNode {
//     bounds: BoundingVolume,
//     sphere_index: u32,
//     num_spheres: u32,
// }

// struct NodeListUniform {
//     num_nodes: u32,
//     list: array<BvhNode>
// }

@group(0) @binding(0)
var<storage> screen: ScreenUniform;

@group(1) @binding(0)
var<storage> spheres: SphereListUniform;

@group(1) @binding(1)
var<storage> planes: PlaneListUniform;

@group(1) @binding(2)
var<storage> aabbs: AabbListUniform;

@group(1) @binding(3)
var<storage> bvh: BvhUniform;

@group(2) @binding(0)
var wavelength_to_xyz_lut: texture_storage_1d<rgba32float, read>;

@group(2) @binding(1)
var rgb_to_spectral_intensity_lut: texture_storage_1d<rgba32float, read>;

@group(2) @binding(2)
var sky_cubemap_texture: texture_cube<f32>;

@group(2) @binding(3)
var sky_cubemap_sampler: sampler;

@group(3) @binding(0)
var color_texture: texture_storage_2d<rgba32float, write>;

@group(3) @binding(1)
var color_texture_copy: texture_storage_2d<rgba32float, read>;

fn raytrace(ray: Ray) -> Hit {
    var closest_hit: Hit;
    var samples = 0.0;

    for (var i = 0u; i < spheres.num_spheres; i++) {
        let sphere = spheres.list[i];

        let hit = ray_sphere_intersect(ray, sphere);
        samples += 1.0;
        closest_hit = merge_hit(closest_hit, hit);
    }

    for (var i = 0u; i < planes.num_planes; i++) {
        let plane = planes.list[i];

        let hit = ray_plane_intersect(ray, plane);
        samples += 1.0;
        closest_hit = merge_hit(closest_hit, hit);
    }

    for (var i = 0u; i < aabbs.num_aabbs; i++) {
        let aabb = aabbs.list[i];

        let hit = ray_aabb_intersect(ray, aabb);
        samples += 1.0;
        closest_hit = merge_hit(closest_hit, hit);
    }

    closest_hit.distance = samples;

    return closest_hit;
}

fn raytrace_bvh(ray: Ray) -> Hit {
    var node_stack = new_node_stack();

    let default_node = BvhNode(
        BoundingVolume(vec3(0.0), vec3(0.0)),
        0, 0, 0
    );

    var closest_hit: Hit;
    var samples = 0.0;

    push_to_node_stack(&node_stack, bvh.nodes[0]);

    while !node_stack_is_empty(&node_stack) {
        let node = top_of_node_stack_or(&node_stack, default_node);
        pop_from_node_stack(&node_stack);


        // Skip to the next node
        if ray_bounding_volume_intersect(ray, node.bounds) > 100000.0 {
            continue;
        }
        samples += 1.0;

        if node.child_node != 0 {
            // node has children, push them to the stack so we can test them next
            push_to_node_stack(&node_stack, bvh.nodes[node.child_node]);
            push_to_node_stack(&node_stack, bvh.nodes[node.child_node + 1]);
        } else {
            // node has no children, trace objects directly
            for (var i = node.start_index; i < node.start_index + node.len; i++) {
                let sphere = spheres.list[i];

                let hit = ray_sphere_intersect(ray, sphere);
                samples += 1.0;
                closest_hit = merge_hit(closest_hit, hit);
            }
        }
    }

    closest_hit.distance = samples;

    return closest_hit;
}

fn debug_raytrace_naive(ray: Ray, tolerance: u32) -> vec3<f32> {
    let hit = raytrace(ray);
    let scale = hit.distance / f32(tolerance);

    if hit.success {
        return vec3(0.0);
    }

    if scale > 1.0 {
        return vec3(1.0, 0.0, 0.0);
    } else {
        return vec3(scale);
    }
}

fn debug_raytrace_bvh(ray: Ray, tolerance: u32) -> vec3<f32> {
    let hit = raytrace_bvh(ray);
    let scale = hit.distance / f32(tolerance);

    if hit.success {
        return vec3(0.0);
    }

    if scale > 1.0 {
        return vec3(1.0, 0.0, 0.0);
    } else {
        return vec3(scale);
    }
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

    init_rng(global_id.xy, screen.view.width, screen.view.height, screen.view.frame_count);

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

    var color = vec3(0.0);

    // let hit = raytrace_bvh(ray);
    // if hit.success {
    //     color = hit.material.albedo;
    // }

    color = debug_raytrace_bvh(ray, spheres.num_spheres);

    // for (var i = 0u; i < 1; i++) {
    //     if ray_bounding_volume_intersect(ray, bvh.nodes[i].bounds) {
    //         color += 0.1;
    //     }
    // }

    // let coord = u32(texcoord.x * 10.0);
    // if bvh.num_nodes == 0 {
    //     color = vec3(1.0, 0.0, 0.0);
    // } else {
    //     color = bvh.nodes[coord].bounds.min;
    // }

    

    textureStore(color_texture, global_id.xy, vec4(color, 1.0));
}