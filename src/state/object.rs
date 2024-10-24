use glam::Vec3;
use gpu_bytes_derive::AsStd430;

use super::material::Material;

const OBJECT_COUNT: usize = 32;
const PAD_THICKNESS: f32 = 0.00025;

#[derive(AsStd430, Default, Debug, Clone, Copy)]
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

#[derive(AsStd430, Default, Debug, Clone, Copy)]
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

#[derive(AsStd430, Default, Debug, Clone, Copy)]
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
}

impl ObjectList {
    pub fn new() -> Self {
        Self {
            spheres: Vec::with_capacity(OBJECT_COUNT),
            planes: Vec::with_capacity(OBJECT_COUNT),
            aabbs: Vec::with_capacity(OBJECT_COUNT),
        }
    }

    pub fn push_sphere(&mut self, sphere: Sphere) {
        self.spheres.insert(0, sphere);
    }

    pub fn push_plane(&mut self, plane: Plane) {
        self.planes.insert(0, plane);
    }

    pub fn push_aabb(&mut self, aabb: Aabb) {
        self.aabbs.insert(0, aabb);
    }
}
