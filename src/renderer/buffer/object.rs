use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::state::object::{Aabb, ObjectList, Plane, Sphere, Triangle};

use super::{DynamicBuffer, UpdateFromSource, MIN_DYNAMIC_BUFFER_CAPACITY};

#[derive(AsStd140, AsStd430)]
pub struct SphereListUniform {
    pub num_spheres: u32,
    pub list: Vec<Sphere>,
}

impl UpdateFromSource<ObjectList> for SphereListUniform {
    fn update(&mut self, object_list: &ObjectList) {
        self.num_spheres = object_list.spheres().len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(object_list.spheres());
    }
}

impl Default for SphereListUniform {
    fn default() -> Self {
        Self {
            num_spheres: 0,
            list: Vec::with_capacity(MIN_DYNAMIC_BUFFER_CAPACITY),
        }
    }
}

#[derive(AsStd140, AsStd430)]
pub struct PlaneListUniform {
    pub num_planes: u32,
    pub list: Vec<Plane>,
}

impl UpdateFromSource<ObjectList> for PlaneListUniform {
    fn update(&mut self, object_list: &ObjectList) {
        self.num_planes = object_list.planes().len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(object_list.planes());
    }
}

impl Default for PlaneListUniform {
    fn default() -> Self {
        Self {
            num_planes: 0,
            list: Vec::with_capacity(MIN_DYNAMIC_BUFFER_CAPACITY),
        }
    }
}

#[derive(AsStd140, AsStd430)]
pub struct AabbListUniform {
    pub num_aabbs: u32,
    pub list: Vec<Aabb>,
}

impl UpdateFromSource<ObjectList> for AabbListUniform {
    fn update(&mut self, object_list: &ObjectList) {
        self.num_aabbs = object_list.aabbs().len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(object_list.aabbs());
    }
}

impl Default for AabbListUniform {
    fn default() -> Self {
        Self {
            num_aabbs: 0,
            list: Vec::with_capacity(MIN_DYNAMIC_BUFFER_CAPACITY),
        }
    }
}

#[derive(AsStd140, AsStd430)]
pub struct TriangleListUniform {
    pub num_triangles: u32,
    pub list: Vec<Triangle>,
}

impl UpdateFromSource<ObjectList> for TriangleListUniform {
    fn update(&mut self, object_list: &ObjectList) {
        self.num_triangles = object_list.triangles().len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(object_list.triangles());
    }
}

impl Default for TriangleListUniform {
    fn default() -> Self {
        Self {
            num_triangles: 0,
            list: Vec::with_capacity(MIN_DYNAMIC_BUFFER_CAPACITY),
        }
    }
}

pub type SphereListBuffer = DynamicBuffer<SphereListUniform, ObjectList>;
pub type PlaneListBuffer = DynamicBuffer<PlaneListUniform, ObjectList>;
pub type AabbListBuffer = DynamicBuffer<AabbListUniform, ObjectList>;
pub type TriangleListBuffer = DynamicBuffer<TriangleListUniform, ObjectList>;
