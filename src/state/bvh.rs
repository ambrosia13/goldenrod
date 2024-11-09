use std::ops::{Add, AddAssign};

use glam::Vec3;
use gpu_bytes_derive::{AsStd140, AsStd430};

use super::object::ObjectList;

pub trait AsBoundingVolume {
    fn bounding_volume(&self) -> BoundingVolume;
}

#[derive(Default, Clone, Copy, Debug, AsStd140, AsStd430)]
pub struct BoundingVolume {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingVolume {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_point(point: Vec3) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    pub fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn grow<T: AsBoundingVolume>(&mut self, object: &T) {
        *self += object.bounding_volume();
    }

    pub fn is_empty(self) -> bool {
        self.min.distance_squared(self.max) < 0.00001
    }
}

impl AsBoundingVolume for BoundingVolume {
    fn bounding_volume(&self) -> BoundingVolume {
        *self
    }
}

impl Add for BoundingVolume {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            min: self.min.min(rhs.min),
            max: self.max.max(rhs.max),
        }
    }
}

impl AddAssign for BoundingVolume {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add<Vec3> for BoundingVolume {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self {
            min: self.min.min(rhs),
            max: self.max.max(rhs),
        }
    }
}

impl AddAssign<Vec3> for BoundingVolume {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = *self + rhs;
    }
}

#[derive(Default, Clone, Copy, AsStd140, AsStd430)]
pub struct BvhNode {
    bounds: BoundingVolume,
    start_index: u32,
    len: u32,
    child_node: u32,
}

impl BvhNode {
    pub fn root<T: AsBoundingVolume>(list: &mut [T]) -> Self {
        let mut bounds = BoundingVolume::new(Vec3::ZERO, Vec3::ZERO);

        for item in list.iter() {
            bounds.grow(item);
        }

        Self {
            // The root node's bounding volume encompasses all objects
            bounds,
            // The root node includes all objects in the list
            start_index: 0,
            len: list.len() as u32,
            // 0 represents no child nodes (yet)
            child_node: 0,
        }
    }

    // returns (axis, threshold)
    fn choose_split_axis(bounds: BoundingVolume) -> (usize, f32) {
        let extent = bounds.max - bounds.min;
        let axis = extent
            .to_array()
            .into_iter()
            .enumerate()
            .max_by(|(_, e1), (_, e2)| e1.total_cmp(e2))
            .map(|(i, _)| i)
            .unwrap();

        let threshold = bounds.center()[axis];

        (axis, threshold)
    }

    pub fn split<T: AsBoundingVolume>(
        &mut self,
        list: &mut [T],
        nodes: &mut Vec<Self>,
        depth: u32,
        max_depth: u32,
    ) {
        if depth == max_depth {
            return;
        }

        if self.len <= 2 {
            return;
        }

        // the child containing objects greater than the split threshold
        let mut child_gt = Self {
            bounds: BoundingVolume::from_point(self.bounds.center()),
            start_index: self.start_index,
            len: 0,
            child_node: 0,
        };

        // the child containing objects less than the split threshold
        let mut child_lt = Self {
            bounds: BoundingVolume::from_point(self.bounds.center()),
            start_index: self.start_index,
            len: 0,
            child_node: 0,
        };

        let (split_axis, split_threshold) = Self::choose_split_axis(self.bounds);
        let greater = |bounds: BoundingVolume| bounds.center()[split_axis] > split_threshold;

        for global_index in self.start_index..(self.start_index + self.len) {
            let global_index = global_index as usize;
            let object = &list[global_index];

            let bounds = object.bounding_volume();

            if greater(bounds) {
                child_gt.bounds.grow(object);
                child_gt.len += 1;

                let swap_index = (child_gt.start_index + child_gt.len) as usize - 1;
                list.swap(swap_index, global_index);
                child_lt.start_index += 1;
            } else {
                child_lt.bounds.grow(object);
                child_lt.len += 1;
            }
        }

        self.child_node = nodes.len() as u32;
        nodes.push(child_gt);
        nodes.push(child_lt);

        // split the children of this node
        child_gt.split(list, nodes, depth + 1, max_depth);
        child_lt.split(list, nodes, depth + 1, max_depth);

        nodes[self.child_node as usize] = child_gt;
        nodes[self.child_node as usize + 1] = child_lt;
    }
}

pub struct BoundingVolumeHierarchy {
    pub version: u32,
    nodes: Vec<BvhNode>,
}

impl BoundingVolumeHierarchy {
    pub fn new<T: AsBoundingVolume>(list: &mut [T], version: u32) -> Self {
        let num_objects_per_leaf = 4;
        let calculated_max_depth = 30;
        //(f32::log2(list.len() as f32 / num_objects_per_leaf as f32) as u32).min(32);

        log::info!(
            "Using a depth of {} for BVH construction",
            calculated_max_depth
        );

        // create the root node
        let mut root = BvhNode::root(list);

        let mut nodes = Vec::with_capacity(1024);
        nodes.push(root);

        root.split(list, &mut nodes, 0, calculated_max_depth);
        nodes[0] = root;

        Self { version, nodes }
    }

    pub fn from_objects(object_list: &mut ObjectList) -> Self {
        // version needs to preemptively incremented because accessing spheres_mut() will increment the version
        let version = object_list.version() + 1;
        Self::new(object_list.triangles_mut(), version)
    }

    pub fn nodes(&self) -> &[BvhNode] {
        &self.nodes
    }
}
