#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/space.wgsl
#include assets/shaders/lib/noise.wgsl
#include assets/shaders/lib/raytrace/stack.wgsl
#include assets/shaders/lib/raytrace/intersect.wgsl

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

@group(0) @binding(0)
var<storage> screen: ScreenUniform;

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

fn sky(ray: Ray) -> vec3<f32> {
    return vec3(0.1, 0.3, 0.95);
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
    brdf: vec3<f32>,
    next_ray: Ray,
}

fn material_hit_result(hit: Hit, ray: Ray, stack: ptr<function, Stack>) -> MaterialHitResult {
    if hit.material.ty == MATERIAL_LAMBERTIAN {
        let brdf = hit.material.albedo / PI;
        let next_ray = Ray(hit.position + hit.normal * 0.0001, ggx_normal(hit.normal, 1.0));

        return MaterialHitResult(brdf, next_ray);
    } else if hit.material.ty == MATERIAL_METAL {
        let rough_normal = ggx_normal(hit.normal, hit.material.roughness);

        let brdf = hit.material.albedo;
        
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

        let cannot_refract = ior * sin_theta > 1.0;

        var brdf = vec3(0.0);
        var pos = hit.position;
        var dir = vec3(0.0);

        if cannot_refract || reflectance(cos_theta, ior) > next_f32() {
            brdf = vec3(1.0);
            
            dir = reflect(ray.dir, rough_normal);
            //dir = mix(dir, generate_cosine_vector(hit.normal), hit.material.roughness);

            pos += hit.normal * 0.0001;
        } else {
            if hit.front_face {
                push_to_stack(stack, current_ior);
            } else {
                pop_from_stack(stack);
            }

            // dir = generate_cosine_vector(hit.normal);
            // pos += hit.normal * 0.0001;

            brdf = hit.material.albedo;

            dir = refract(ray.dir, rough_normal, ior);
            //dir = normalize(dir + generate_unit_vector() * hit.material.roughness);

            pos -= hit.normal * 0.0001;
        }


        return MaterialHitResult(brdf, Ray(pos, dir));
    } else {
        return MaterialHitResult(vec3(0.0), Ray(vec3(0.0), vec3(0.0)));
    }
}

fn pathtrace(ray: Ray) -> vec3<f32> {
    var incoming_normal = vec3(10.0);
    var ior_stack = new_stack();

    var throughput = vec3(1.0);
    var radiance = vec3(0.0);

    var current_ray = ray;

    var bounces = 5;
    
    let should_accumulate = 
        all(screen.camera.position == screen.camera.previous_position) &&
        all(screen.camera.view == screen.camera.previous_view);

    if should_accumulate {
        bounces = 50;
    }

    for (var i = 0; i < bounces; i++) {
        var hit: Hit;
        var weight: f32 = 1.0 / TAU;

        hit = raytrace(current_ray);

        if !hit.success {
            // hit sky
            radiance += throughput * sky(current_ray);
            break;
        }

        incoming_normal = hit.normal;
        radiance += throughput * hit.material.emission;

        let material_hit_result = material_hit_result(hit, current_ray, &ior_stack);
        throughput *= material_hit_result.brdf;

        current_ray = material_hit_result.next_ray;
    }

    return radiance;
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

    let rays = 1;
    for (var i = 0; i < rays; i++) {
        color += pathtrace(ray) / f32(rays);
    }

    let sample = textureLoad(color_texture_copy, global_id.xy);
    let previous_color = sample.rgb;
    var frame_age = sample.a;

    let should_accumulate = 
        all(screen.camera.position == screen.camera.previous_position) &&
        all(screen.camera.view == screen.camera.previous_view) && 
        all(screen.camera.projection_matrix[0] == screen.camera.previous_projection_matrix[0]);
    
    if !should_accumulate {
        frame_age = 0.0;
    }

    color = mix(previous_color, color, 1.0 / (frame_age + 1.0));

    textureStore(color_texture, global_id.xy, vec4(color, frame_age + 1.0));
}