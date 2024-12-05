#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/space.wgsl
#include assets/shaders/lib/noise.wgsl
#include assets/shaders/lib/raytrace/stack.wgsl
#include assets/shaders/lib/raytrace/intersect.wgsl
#include assets/shaders/lib/raytrace/spectrum.wgsl
#include assets/shaders/lib/raytrace/bvh.wgsl

const IOR_AIR: f32 = 1.000293;

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

struct TriangleListUniform {
    num_triangles: u32,
    list: array<Triangle>,
}

struct BvhUniform {
    num_nodes: u32,
    nodes: array<BvhNode>,
}

@group(0) @binding(0)
var<storage> screen: ScreenUniform;

@group(1) @binding(0)
var<storage> spheres: SphereListUniform;

@group(1) @binding(1)
var<storage> planes: PlaneListUniform;

@group(1) @binding(2)
var<storage> aabbs: AabbListUniform;

@group(1) @binding(3)
var<storage> triangles: TriangleListUniform;

@group(1) @binding(4)
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

fn albedo(hit: Hit) -> vec3<f32> {
    //if any(hit.uv < vec2(0.0)) {
        return hit.material.albedo;
    //} else {
        //return vec3(clamp(hit.uv, vec2(0.0), vec2(1.0)), 0.01);
    //}
}

fn sky(ray: Ray) -> vec3<f32> {
    var color = textureSampleLevel(sky_cubemap_texture, sky_cubemap_sampler, ray.dir, 0.0).rgb;
    color = pow(color, vec3(2.2));
    let color_length = length(color);

    color = normalize(color) * min(1000.0, color_length);

    return color;
}

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

fn raytrace_bvh(ray: Ray) -> Hit {
    var node_stack = new_node_stack();
    let default_node = 0u;

    var closest_hit: Hit;

    push_to_node_stack(&node_stack, 0u);

    while !node_stack_is_empty(&node_stack) {
        let node_index = top_of_node_stack_or(&node_stack, default_node);
        let node = bvh.nodes[node_index];

        pop_from_node_stack(&node_stack);

        let node_hit = ray_bounding_volume_intersect(ray, node.bounds);

        // // Skip to the next node if we don't intersect the current node's bounding box
        // // or if we've already found a hit closer than the bounding box
        // if !node_hit.success || (closest_hit.success && closest_hit.distance < node_hit.distance) {
        //     continue;
        // }

        if node.child_node != 0 {
            // node has children, push them to the stack so we can test them next
            let bounds_a = bvh.nodes[node.child_node].bounds;
            let bounds_b = bvh.nodes[node.child_node + 1].bounds;

            let child_a_hit = ray_bounding_volume_intersect(ray, bounds_a);
            let child_b_hit = ray_bounding_volume_intersect(ray, bounds_b);

            // Push the first child if we're hitting its bounding volume, and if we haven't already found a closer triangle hit
            var push_child_a = child_a_hit.success && !(closest_hit.success && closest_hit.distance < child_a_hit.distance);

            // Push the second child if we're hitting its bounding volume, and if we haven't already found a closer triangle hit
            var push_child_b = child_b_hit.success && !(closest_hit.success && closest_hit.distance < child_b_hit.distance);

            if push_child_a && push_child_b {
                if child_a_hit.distance < child_b_hit.distance {
                    push_to_node_stack(&node_stack, node.child_node + 1);
                    push_to_node_stack(&node_stack, node.child_node);
                } else {
                    push_to_node_stack(&node_stack, node.child_node);
                    push_to_node_stack(&node_stack, node.child_node + 1);
                }
            } else if push_child_a {
                push_to_node_stack(&node_stack, node.child_node);
            } else if push_child_b {
                push_to_node_stack(&node_stack, node.child_node + 1);
            }
        } else {
            // node has no children, trace objects directly
            for (var i = node.start_index; i < node.start_index + node.len; i++) {
                let triangle = triangles.list[i];

                let hit = ray_triangle_intersect(ray, triangle);
                closest_hit = merge_hit(closest_hit, hit);
            }
        }
    }

    return closest_hit;
}

fn raytrace_all(ray: Ray) -> Hit {
    var hit: Hit;
    hit = merge_hit(hit, raytrace(ray));
    hit = merge_hit(hit, raytrace_bvh(ray));
    return hit;
}

// Schlick approximation for reflectance
fn reflectance(cos_theta: f32, ior: f32) -> f32 {
    var r0 = (1.0 - ior) / (1.0 + ior);
    r0 *= r0;

    return r0 + (1.0 - r0) * pow(1.0 - cos_theta, 5.0);
}

fn ggx_normal(normal: vec3<f32>, roughness: f32) -> vec3<f32> {
    let r1 = next_f32();
    let r2 = next_f32();

    let theta = acos(sqrt(r1));
    let phi = 2.0 * PI * r2;

    let x = sin(theta) * cos(phi);
    let y = sin(theta) * sin(phi);
    let z = cos(theta);

    let rough_normal = vec3(x, y, z);

    return mix(normal, normalize(tbn_matrix(normal) * rough_normal), roughness);
}

struct MaterialHitResult {
    brdf: f32,
    next_ray: Ray,
}

fn material_hit_result(hit: Hit, ray: Ray, stack: ptr<function, Stack>, wavelength: f32) -> MaterialHitResult {
    let albedo = rgb_to_spectral_intensity(rgb_to_spectral_intensity_lut, albedo(hit), wavelength);
    
    if hit.material.ty == MATERIAL_LAMBERTIAN {
        let brdf = albedo / PI;
        let next_ray = Ray(hit.position + hit.normal * 0.0001, ggx_normal(hit.normal, 1.0));

        return MaterialHitResult(brdf, next_ray);
    } else if hit.material.ty == MATERIAL_METAL {
        let rough_normal = ggx_normal(hit.normal, hit.material.roughness);

        let brdf = albedo;
        
        let reflect_dir = reflect(ray.dir, rough_normal);
        let next_ray = Ray(
            hit.position + hit.normal * 0.0001, 
            reflect_dir
        );

        return MaterialHitResult(brdf, next_ray);
    } else if hit.material.ty == MATERIAL_DIELECTRIC {
        let rough_normal = ggx_normal(hit.normal, hit.material.roughness);

        let cos_theta = dot(-ray.dir, rough_normal);
        let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

        let previous_ior = top_of_stack_or(stack, IOR_AIR);
        let current_ior = hit.material.ior;

        var ior: f32;

        if hit.front_face {
            ior = previous_ior / current_ior;
        } else {
            ior = current_ior / previous_ior;
        }

        ior = ior - ((wavelength - 550.0) * 0.0001);

        let cannot_refract = ior * sin_theta > 1.0;

        var brdf = 0.0;
        var pos = hit.position;
        var dir = vec3(0.0);

        if cannot_refract || reflectance(cos_theta, ior) > next_f32() {
            brdf = 1.0;
            
            dir = reflect(ray.dir, rough_normal);
            pos += hit.normal * 0.0001;
        } else {
            if hit.front_face {
                push_to_stack(stack, current_ior);
            } else {
                pop_from_stack(stack);
            }

            brdf = albedo;

            dir = refract(ray.dir, rough_normal, ior);
            pos -= hit.normal * 0.0001;
        }

        return MaterialHitResult(brdf, Ray(pos, dir));
    // } else if hit.material.ty == MATERIAL_VOLUME {
    //     var distance_through_volume = 0.0;

    //     if hit.front_face {
    //         distance_through_volume = hit.far_distance - hit.distance;
    //     } else {
    //         distance_through_volume = hit.distance;
    //     }

    //     let density = 0.05;
    //     let scatter_distance = -log(next_f32()) / density;
    //     let scatter = scatter_distance < distance_through_volume;

    //     if scatter {
    //         let brdf = albedo;
    //         let ray_dir = mix(generate_unit_vector(), ray.dir, 0.0);
    //         let scattered_ray = Ray(hit.position + ray_dir * 0.0001, ray_dir);
    //         return MaterialHitResult(albedo, scattered_ray);
    //     } else {
    //         let transmittance = exp(-density * distance_through_volume);
    //         let brdf = 1.0 * transmittance;
            
    //         let transmitted_ray = Ray(hit.position + ray.dir * 0.0001, ray.dir);
    //         return MaterialHitResult(brdf, transmitted_ray);
    //     }
    } else {
        return MaterialHitResult(0.0, Ray(vec3(0.0), vec3(0.0)));
    }
}

fn pathtrace(ray: Ray, wavelength: f32) -> vec3<f32> {
    var incoming_normal = vec3(10.0);
    var ior_stack = new_stack();

    var throughput = 1.0;
    var radiance = 0.0;

    var current_ray = ray;

    let max_bounces = 100;

    for (var i = 0; i < max_bounces; i++) {
        let hit = raytrace_all(current_ray);

        if !hit.success {
            // hit sky
            radiance += throughput * rgb_to_spectral_intensity(rgb_to_spectral_intensity_lut, sky(current_ray), wavelength);
            break;
        }

        incoming_normal = hit.normal;
        radiance += throughput * rgb_to_spectral_intensity(rgb_to_spectral_intensity_lut, hit.material.emission, wavelength);

        let material_hit_result = material_hit_result(hit, current_ray, &ior_stack, wavelength);
        throughput *= material_hit_result.brdf;

        // Russian roulette path termination
        let probability = clamp(throughput, 0.0, 1.0);
        if next_f32() > probability {
            break;
        }

        // account for the energy lost by rays that were terminated
        throughput *= 1.0 / probability;

        current_ray = material_hit_result.next_ray;
    }

    return xyz_to_rgb(wavelength_to_xyz(wavelength_to_xyz_lut, wavelength)) * radiance;
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

    let should_accumulate = 
        all(screen.camera.position == screen.camera.previous_position) &&
        all(screen.camera.view == screen.camera.previous_view) && 
        all(screen.camera.projection_matrix[0] == screen.camera.previous_projection_matrix[0]);

    let sample = textureLoad(color_texture_copy, global_id.xy);
    let previous_color = sample.rgb;
    var frame_age = sample.a;

    
    if !should_accumulate {
        frame_age = 0.0;
    }

    let debug_render_in_movement = false;
    let accumulation_threshold = 100.0;

    // Path trace
    if !debug_render_in_movement || frame_age > accumulation_threshold  {
        let rays = 1;
        for (var i = 0; i < rays; i++) {
            let wavelength = generate_wavelength();
            color += pathtrace(ray, wavelength) / f32(rays);
        }

        if !debug_render_in_movement || frame_age > accumulation_threshold + 1.0 {
            var accumulated_frame_age = frame_age - accumulation_threshold;

            if !debug_render_in_movement {
                accumulated_frame_age = frame_age;
            }

            color = mix(previous_color, color, 1.0 / (accumulated_frame_age + 1.0));
        }
    }
    // Debug render 
    else {
        let hit = raytrace_all(ray);
        if hit.success {
            let shading_factor = max(0.0, dot(hit.normal, vec3(0.0, 1.0, 0.0))) * 0.5 + 0.5;
            color = hit.material.albedo * shading_factor;
        } else {
            color = sky(ray);
        }
    }

    textureStore(color_texture, global_id.xy, vec4(color, frame_age + 1.0));
}