use core::f32;

use glam::{Quat, Vec2, Vec3};
use gpu_bytes::{AsStd430, Std430Bytes};
use gpu_bytes_derive::{AsStd140, AsStd430};
use rand::Rng;

use crate::util;

use super::{
    bvh::{AsBoundingVolume, BoundingVolume},
    material::{Material, MaterialType},
};

const PAD_THICKNESS: f32 = 0.00025;

#[derive(AsStd140, AsStd430, Default, Debug, Clone, Copy)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn pad(self) -> Self {
        Self {
            radius: self.radius - PAD_THICKNESS,
            ..self
        }
    }
}

impl AsBoundingVolume for Sphere {
    fn bounding_volume(&self) -> BoundingVolume {
        BoundingVolume {
            min: self.center - self.radius,
            max: self.center + self.radius,
        }
    }
}

#[derive(AsStd140, AsStd430, Default, Debug, Clone, Copy)]
pub struct Plane {
    normal: Vec3,
    point: Vec3,
    material: Material,
}

impl Plane {
    pub fn new(normal: Vec3, point: Vec3, material: Material) -> Self {
        Self {
            normal,
            point,
            material,
        }
    }
}

#[derive(AsStd140, AsStd430, Default, Debug, Clone, Copy)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
    material: Material,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Self { min, max, material }
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn pad(self) -> Self {
        Self {
            min: self.min + PAD_THICKNESS,
            max: self.max - PAD_THICKNESS,
            ..self
        }
    }
}

impl AsBoundingVolume for Aabb {
    fn bounding_volume(&self) -> BoundingVolume {
        BoundingVolume {
            min: self.min,
            max: self.max,
        }
    }
}

#[derive(AsStd140, Default, Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub uv_a: Vec2,
    pub uv_b: Vec2,
    pub uv_c: Vec2,
    pub material: Material,
    pub bounds: BoundingVolume,
}

impl AsStd430 for Triangle {
    fn as_std430(&self) -> gpu_bytes::Std430Bytes {
        let mut buf = Std430Bytes::new();

        buf.write(&self.a);
        buf.write(&self.b);
        buf.write(&self.c);
        buf.write(&self.uv_a);
        buf.write(&self.uv_b);
        buf.write(&self.uv_c);
        buf.write(&self.material);

        buf.align();

        buf
    }
}

impl Triangle {
    pub fn new(
        a: Vec3,
        b: Vec3,
        c: Vec3,
        uv_a: Vec2,
        uv_b: Vec2,
        uv_c: Vec2,
        material: Material,
    ) -> Self {
        Self {
            a,
            b,
            c,
            uv_a,
            uv_b,
            uv_c,
            material,
            bounds: BoundingVolume {
                min: a.min(b.min(c)),
                max: a.max(b.max(c)),
            },
        }
    }

    pub fn vertices(&self) -> [Vec3; 3] {
        [self.a, self.b, self.c]
    }
}

impl AsBoundingVolume for Triangle {
    fn bounding_volume(&self) -> BoundingVolume {
        self.bounds
    }
}

pub struct ObjectList {
    spheres: Vec<Sphere>,
    planes: Vec<Plane>,
    aabbs: Vec<Aabb>,
    triangles: Vec<Triangle>,

    version: u32,
}

impl ObjectList {
    pub fn new() -> Self {
        Self {
            spheres: Vec::new(),
            planes: Vec::new(),
            aabbs: Vec::new(),
            triangles: Vec::new(),
            version: 0,
        }
    }

    pub fn cubeception(&mut self, albedo: Vec3, position: Vec3, radius: f32, ior: f32, depth: u32) {
        self.version += 1;

        let material = Material {
            albedo,
            ty: MaterialType::Dielectric,
            emission: Vec3::ZERO,
            roughness: 0.0,
            ior,
            g: 0.0,
        };

        let mut radius = radius;

        for i in 0..depth {
            if i % 2 == 0 {
                // AABB
                self.push_aabb(Aabb {
                    min: position - Vec3::splat(radius),
                    max: position + Vec3::splat(radius),
                    material,
                });
            } else {
                // Sphere
                self.push_sphere(Sphere {
                    center: position,
                    radius,
                    material,
                });

                // calculate the radius of the next aabb
                radius /= f32::sqrt(3.0);
            }
        }
    }

    pub fn random_spheres(&mut self, count: u32) {
        self.version += 1;

        self.spheres.clear();
        self.planes.clear();
        self.aabbs.clear();

        let center = Vec3::new(0.0, 30.0, 0.0);

        // self.push_sphere(Sphere::new(
        //     center,
        //     7.5,
        //     Material {
        //         albedo: Vec3::ONE,
        //         ty: MaterialType::Volume,
        //         g: 0.75,
        //         ..Default::default()
        //     },
        // ));

        self.push_sphere(Sphere::new(
            center,
            5.0,
            Material::lambertian(Vec3::new(0.5, 1.0, 0.2)),
        ));

        let mut rng = rand::thread_rng();

        let range = 20.0;

        for i in 0..count {
            let center = Vec3::new(
                rng.gen_range(-range..range),
                rng.gen_range(-range..range),
                rng.gen_range(-(range * 0.25)..(range * 0.25)),
            );

            let radius = 1.0;

            self.push_sphere(Sphere::new(center, radius, Material::random()));
        }
    }

    pub fn bvh_test_scene(&mut self) {
        self.version += 1;

        self.spheres.clear();
        self.planes.clear();
        self.aabbs.clear();
        self.triangles.clear();

        let triangles = util::gltf::load_triangles_from_glb(
            "assets/meshes/car.glb",
            Vec3::new(0.0, -1.5, -0.25),
            Quat::from_rotation_y(f32::consts::PI) * Quat::from_rotation_x(-f32::consts::PI / 2.0),
            0.1,
            Material::metal(Vec3::new(1.0, 0.75, 0.2), 0.05),
        )
        .unwrap();

        self.triangles.extend_from_slice(&triangles);
    }

    pub fn random_scene(&mut self) {
        self.version += 1;

        self.spheres.clear();
        self.planes.clear();
        self.aabbs.clear();
        self.triangles.clear();

        self.push_plane(Plane::new(
            Vec3::Y,
            Vec3::ZERO - PAD_THICKNESS * 2.5,
            Material {
                ty: MaterialType::Lambertian,
                albedo: Vec3::ONE,
                emission: Vec3::ZERO,
                roughness: 0.0,
                ior: 0.0,
                g: 0.0,
            },
        ));

        self.push_plane(Plane::new(
            Vec3::Y,
            Vec3::ZERO,
            Material {
                ty: MaterialType::Dielectric,
                albedo: Vec3::ONE,
                emission: Vec3::ZERO,
                roughness: 0.1,
                ior: 1.05,
                g: 0.0,
            },
        ));

        let region_size = 7;
        let regions_radius = 2;

        for x in -regions_radius..=regions_radius {
            for z in -regions_radius..=regions_radius {
                let x = (x * region_size) as f32;
                let z = (z * region_size) as f32;

                let max_offset = region_size as f32 / 2.0 * 0.8;
                let min_radius = region_size as f32 / 2.0 * 0.2;

                let offset_x = rand::thread_rng().gen_range(-max_offset..=max_offset);
                let offset_z = rand::thread_rng().gen_range(-max_offset..=max_offset);

                let max_radius = max_offset - offset_x.abs().max(offset_z.abs());

                let rand_radius = || {
                    rand::thread_rng()
                        .gen_range(0.0..=max_radius)
                        .max(min_radius)
                        .sqrt()
                };

                match rand::thread_rng().gen_range(0..3) {
                    0 => {
                        let radius = rand_radius();

                        self.push_sphere(
                            Sphere::new(
                                Vec3::new(x + offset_x, radius, z + offset_z),
                                radius,
                                Material::random(),
                            )
                            .pad(),
                        )
                    }
                    1 => {
                        let radius_x = rand_radius();
                        let radius_y = rand_radius();
                        let radius_z = rand_radius();

                        self.push_aabb(
                            Aabb::new(
                                Vec3::new(x + offset_x - radius_x, 0.0, z + offset_z - radius_z),
                                Vec3::new(
                                    x + offset_x + radius_x,
                                    2.0 * radius_y,
                                    z + offset_z + radius_z,
                                ),
                                Material::random(),
                            )
                            .pad(),
                        )
                    }
                    2 => {
                        // let scale = rand_radius();
                        // let angle = rand::thread_rng().gen_range(0.0..f32::consts::TAU);
                        // let rotation = Quat::from_rotation_y(angle);

                        // let triangles = util::gltf::load_triangles_from_gltf(
                        //     "assets/meshes/suzanne",
                        //     Vec3::new(x + offset_x, scale, z + offset_z),
                        //     rotation,
                        //     scale,
                        //     Material::random(),
                        // )
                        // .unwrap();

                        // for triangle in triangles {
                        //     self.push_triangle(triangle);
                        // }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn push_sphere(&mut self, sphere: Sphere) {
        self.version += 1;
        self.spheres.push(sphere);
    }

    pub fn push_plane(&mut self, plane: Plane) {
        self.version += 1;
        self.planes.push(plane);
    }

    pub fn push_aabb(&mut self, aabb: Aabb) {
        self.version += 1;
        self.aabbs.push(aabb);
    }

    pub fn push_triangle(&mut self, triangle: Triangle) {
        self.version += 1;
        self.triangles.push(triangle);
    }

    pub fn spheres(&self) -> &[Sphere] {
        &self.spheres
    }

    pub fn spheres_mut(&mut self) -> &mut [Sphere] {
        self.version += 1;
        &mut self.spheres
    }

    pub fn planes(&self) -> &[Plane] {
        &self.planes
    }

    pub fn aabbs(&self) -> &[Aabb] {
        &self.aabbs
    }

    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn triangles_mut(&mut self) -> &mut [Triangle] {
        self.version += 1;
        &mut self.triangles
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}
