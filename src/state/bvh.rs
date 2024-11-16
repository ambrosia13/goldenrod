use std::ops::{Add, AddAssign};

use glam::{FloatExt, Vec3};
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

    pub fn surface_area(self) -> f32 {
        let extent = self.max - self.min;

        let width = extent.x;
        let height = extent.y;
        let depth = extent.z;

        2.0 * (width * height + width * depth + height * depth)
    }

    // pub fn split_along_axis(self, axis: usize, midpoint: Vec3) {
    //     let mut split = Vec3::new(midpoint);
    // }

    pub fn grow<T: AsBoundingVolume>(&mut self, object: &T) {
        let bounds = object.bounding_volume();

        self.min = self.min.min(bounds.min);
        self.max = self.max.max(bounds.max);
    }

    pub fn shrink<T: AsBoundingVolume>(&mut self, object: &T) {
        let bounds = object.bounding_volume();

        self.min = self.min.max(bounds.min);
        self.max = self.max.min(bounds.max);
    }

    pub fn is_empty(self) -> bool {
        self.min.distance_squared(self.max) < 0.00001
    }
}

#[test]
fn test() {
    let mut bounds = BoundingVolume::new(Vec3::ZERO, Vec3::ONE);
    bounds.shrink(&BoundingVolume::new(Vec3::ONE * 0.8, Vec3::ONE * 1.5));

    println!("{:?}", bounds);
}

impl AsBoundingVolume for BoundingVolume {
    fn bounding_volume(&self) -> BoundingVolume {
        *self
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

    pub fn slice<T>(self, list: &[T]) -> &[T] {
        let start = self.start_index as usize;
        let end = start + self.len as usize;
        &list[start..end]
    }

    fn split_cost<T: AsBoundingVolume>(
        bounds: BoundingVolume,
        list: &[T],
        axis: usize,
        threshold: f32,
    ) -> f32 {
        let mut bounds_a = BoundingVolume::from_point(bounds.min);
        let mut bounds_b = BoundingVolume::from_point(bounds.max);

        let mut a_count = 0;
        let mut b_count = 0;

        for obj in list {
            let obj_bounds = obj.bounding_volume();
            let obj_center = obj_bounds.center();

            if obj_center[axis] < threshold {
                bounds_a.grow(obj);
                a_count += 1;
            } else {
                bounds_b.grow(obj);
                b_count += 1;
            }
        }

        let node_cost = 1.0;
        let object_cost = 1.0;

        let a_cost = bounds_a.surface_area() * a_count as f32 * object_cost;
        let b_cost = bounds_b.surface_area() * b_count as f32 * object_cost;
        node_cost + a_cost + b_cost
    }

    // returns (axis, threshold)
    fn choose_split_axis<T: AsBoundingVolume + Clone>(
        bounds: BoundingVolume,
        list: &[T],
    ) -> (usize, f32) {
        // let extent = bounds.max - bounds.min;
        // let axis = extent
        //     .to_array()
        //     .into_iter()
        //     .enumerate()
        //     .max_by(|(_, e1), (_, e2)| e1.total_cmp(e2))
        //     .map(|(i, _)| i)
        //     .unwrap();

        // let mut sorted_list: Vec<_> = list_iter.collect();
        // sorted_list.sort_by(|a, b| {
        //     a.bounding_volume().center()[axis].total_cmp(&b.bounding_volume().center()[axis])
        // });

        // let half_len = sorted_list.len() / 2;
        // let threshold = sorted_list[half_len].bounding_volume().center()[axis] + 0.01;

        // (axis, threshold)

        let search_steps = 20.min(list.len() + 1);

        let mut best_cost = f32::INFINITY;
        let mut best_axis = 0;
        let mut best_threshold = 0.0;

        for axis in 0..3 {
            // let mut sorted_objects = list.to_vec();

            // sorted_objects.sort_by(|a, b| {
            //     let a_center = a.bounding_volume().center();
            //     let b_center = b.bounding_volume().center();

            //     a_center[axis].total_cmp(&b_center[axis])
            // });

            let bounds_start = bounds.min[axis];
            let bounds_end = bounds.max[axis];

            for i in 0..search_steps {
                let threshold =
                    bounds_start.lerp(bounds_end, (i as f32 + 0.5) / search_steps as f32);

                let cost = Self::split_cost(bounds, list, axis, threshold);

                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_threshold = threshold;
                }
            }
        }

        (best_axis, best_threshold)
    }

    pub fn split<T: AsBoundingVolume + Clone>(
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

        let (split_axis, split_threshold) = Self::choose_split_axis(self.bounds, self.slice(list));

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
    pub fn new<T: AsBoundingVolume + Clone>(list: &mut [T], version: u32) -> Self {
        let max_depth = f32::log2(list.len() as f32) as u32;

        // create the root node
        let mut root = BvhNode::root(list);

        let mut nodes = Vec::with_capacity(1024);
        nodes.push(root);

        if !list.is_empty() {
            root.split(list, &mut nodes, 0, max_depth);
            nodes[0] = root;
        }

        let leaf_node_count = nodes.iter().filter(|node| node.child_node == 0).count();

        let largest_leaf_object_count = nodes
            .iter()
            .skip(1)
            .filter(|node| node.child_node == 0)
            .map(|node| node.len)
            .max()
            .unwrap();

        fn get_max_depth(nodes: &[BvhNode], index: usize) -> u32 {
            let node = nodes[index];

            if node.child_node == 0 {
                // no children, stop the count
                1
            } else {
                let child_node_index = node.child_node as usize;

                // get the maximum effective depth of the two children
                1 + get_max_depth(nodes, child_node_index)
                    .max(get_max_depth(nodes, child_node_index + 1))
            }
        }

        log::info!(
            r#"
            BVH: Object count: {},
            BVH: Number of nodes: {},
            BVH: Node max depth: {},
            BVH: Actual node max depth: {},
            BVH: Number of leaf nodes: {},
            BVH: Largest leaf object count: {}
            "#,
            list.len(),
            nodes.len(),
            max_depth,
            get_max_depth(&nodes, 0), // start search with the root node
            leaf_node_count,
            largest_leaf_object_count
        );

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
