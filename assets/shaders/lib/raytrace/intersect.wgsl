struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}

struct Hit {
    success: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    distance: f32,
    front_face: bool,
    material: Material,
}

const MATERIAL_LAMBERTIAN: u32 = 0u;
const MATERIAL_METAL: u32 = 1u;
const MATERIAL_DIELECTRIC: u32 = 2u;

struct Material {
    albedo: vec3<f32>,
    ty: u32,
    emission: vec3<f32>,
    roughness: f32,
    ior: f32,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material,
}

struct Plane {
    normal: vec3<f32>,
    point: vec3<f32>,
    material: Material,
}

struct Aabb {
    min: vec3<f32>,
    max: vec3<f32>,
    material: Material,
}

fn merge_hit(a: Hit, b: Hit) -> Hit {
    var hit: Hit;

    if !(a.success || b.success) {
        hit.success = false;
        return hit;
    } else if a.success && !b.success {
        return a;
    } else if b.success && !a.success {
        return b;
    } else {
        if a.distance < b.distance {
            hit = a;
        } else {
            hit = b;
        }
    }

    return hit;
}

fn ray_sphere_intersect(ray: Ray, sphere: Sphere) -> Hit {
    var hit: Hit;
    hit.success = false;
    hit.material = sphere.material;

    let origin_to_center = ray.pos - sphere.center;

    let b = dot(origin_to_center, ray.dir);
    let a = dot(ray.dir, ray.dir);
    let c = dot(origin_to_center, origin_to_center) - sphere.radius * sphere.radius;

    let determinant = b * b - a * c;

    if determinant >= 0.0 {
        let determinant_sqrt = sqrt(determinant);
        var t = (-b - determinant_sqrt) / a;

        t = mix(t, (-b + determinant_sqrt) / a, f32(t < 0.0));

        // if t < 0.0 {
        //     t = (-b + determinant_sqrt) / a;
        // }

        if t >= 0.0 {
            let point = ray.pos + ray.dir * t;
            let outward_normal = normalize(point - sphere.center);

            let dir_dot_normal = dot(ray.dir, outward_normal);
            let front_face = dir_dot_normal < 0.0;

            var normal = outward_normal * -sign(dir_dot_normal);

            hit.success = true;
            hit.position = point;
            hit.normal = normal;
            hit.distance = t;
            hit.front_face = front_face;
        }
    }

    return hit;
}

fn ray_plane_intersect(ray: Ray, plane: Plane) -> Hit {
    var hit: Hit;
    hit.success = false;
    hit.material = plane.material;

    let denom = dot(plane.normal, ray.dir);

    if abs(denom) < 1e-6 {
        return hit;
    }

    let t = dot(plane.normal, plane.point - ray.pos) / denom;

    if t < 0.0 {
        return hit;
    }

    hit.success = true;
    hit.position = ray.pos + ray.dir * t;
    hit.normal = plane.normal * -sign(denom);
    hit.distance = t;
    hit.front_face = true;

    return hit;
}

fn ray_aabb_intersect(ray: Ray, aabb: Aabb) -> Hit {
    var hit: Hit;
    hit.material = aabb.material;
    hit.front_face = !all(clamp(ray.pos, aabb.min, aabb.max) == ray.pos);

    let t_min = (aabb.min - ray.pos) / ray.dir;
    let t_max = (aabb.max - ray.pos) / ray.dir;

    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);

    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);

    if !hit.front_face { // ray inside box
        hit.success = true;
        hit.distance = t_far;     
        
        let eq = t2 == vec3(t_far);
        hit.normal = vec3(f32(eq.x), f32(eq.y), f32(eq.z)) * -sign(ray.dir);   
    } else {
        hit.success = !(t_near > t_far || t_far < 0.0);
        hit.distance = t_near;

        let eq = t1 == vec3(t_near);
        hit.normal = vec3(f32(eq.x), f32(eq.y), f32(eq.z)) * -sign(ray.dir);
    }

    hit.position = ray.pos + ray.dir * hit.distance;

    return hit;
}