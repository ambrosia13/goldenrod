use glam::Vec3;
use gpu_bytes_derive::{AsStd140, AsStd430};
use rand::Rng;

use super::material::{Material, MaterialType};

const OBJECT_COUNT: usize = 32;
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

pub struct ObjectList {
    pub spheres: Vec<Sphere>,
    pub planes: Vec<Plane>,
    pub aabbs: Vec<Aabb>,

    pub version: u32,
}

impl ObjectList {
    pub fn new() -> Self {
        Self {
            spheres: Vec::new(),
            planes: Vec::new(),
            aabbs: Vec::new(),
            version: 0,
        }
    }

    pub fn random_scene(&mut self) {
        self.version += 1;

        self.spheres.clear();
        self.planes.clear();
        self.aabbs.clear();

        self.planes.push(Plane::new(
            Vec3::Y,
            Vec3::ZERO,
            Material {
                ty: MaterialType::Lambertian,
                albedo: Vec3::ONE,
                emission: Vec3::ZERO,
                roughness: 0.0,
                ior: 0.0,
            },
        ));

        let region_size = 5;
        let regions_radius = 2;

        for x in -regions_radius..=regions_radius {
            for z in -regions_radius..=regions_radius {
                let x = (x * region_size) as f32;
                let z = (z * region_size) as f32;

                let max_offset = region_size as f32 / 2.0 * 0.8;
                let min_radius = region_size as f32 / 2.0 * 0.2;

                let offset = rand::thread_rng().gen_range(-max_offset..=max_offset);

                let rand_radius = || {
                    rand::thread_rng()
                        .gen_range(min_radius..=(max_offset - offset.abs() + min_radius))
                        .sqrt()
                };

                match rand::thread_rng().gen_range(0..2) {
                    0 => {
                        let radius = rand_radius();

                        self.push_sphere(
                            Sphere::new(
                                Vec3::new(x + offset, radius, z + offset),
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
                                Vec3::new(x + offset - radius_x, 0.0, z + offset - radius_z),
                                Vec3::new(
                                    x + offset + radius_x,
                                    2.0 * radius_y,
                                    z + offset + radius_z,
                                ),
                                Material::random(),
                            )
                            .pad(),
                        )
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
}
