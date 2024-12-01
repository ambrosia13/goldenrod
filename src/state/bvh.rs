use glam::Vec3;
use gpu_bytes::{AsStd140, AsStd430, Std140Bytes, Std430Bytes};
use gpu_bytes_derive::{AsStd140, AsStd430};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::object::ObjectList;

pub trait AsBoundingVolume {
    fn bounding_volume(&self) -> BoundingVolume;

    fn center(&self) -> Vec3 {
        self.bounding_volume().center()
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct BoundingVolume {
    pub min: Vec3,
    pub max: Vec3,
    pub empty: bool,
}

impl AsStd140 for BoundingVolume {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write(&self.min);
        buf.write(&self.max);
        buf.align();

        buf
    }
}

impl AsStd430 for BoundingVolume {
    fn as_std430(&self) -> Std430Bytes {
        let mut buf = Std430Bytes::new();

        buf.write(&self.min);
        buf.write(&self.max);
        buf.align();

        buf
    }
}

impl BoundingVolume {
    pub const EMPTY: Self = Self {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
        empty: true,
    };

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
            empty: false,
        }
    }

    pub fn from_point(point: Vec3) -> Self {
        Self {
            min: point,
            max: point,
            empty: false,
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

    pub fn grow<T: AsBoundingVolume>(&mut self, object: &T) {
        let bounds = object.bounding_volume();

        if !self.empty {
            self.min = self.min.min(bounds.min);
            self.max = self.max.max(bounds.max);
        } else {
            *self = bounds;
        }
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

#[derive(Default, Clone, Copy, AsStd140, AsStd430)]
pub struct BvhNode {
    bounds: BoundingVolume,
    start_index: u32,
    len: u32,
    child_node: u32,
}

impl BvhNode {
    pub const NODE_COST: f32 = 0.0;
    pub const OBJECT_COST: f32 = 2.0;

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

    fn cost(&self) -> f32 {
        Self::NODE_COST + Self::OBJECT_COST * self.bounds.surface_area() * self.len as f32
    }

    fn evaluate_split_cost<T: AsBoundingVolume>(list: &[T], axis: usize, threshold: f32) -> f32 {
        let mut bounds_a = BoundingVolume::EMPTY;
        let mut bounds_b = BoundingVolume::EMPTY;

        let mut a_count = 0;
        let mut b_count = 0;

        for obj in list {
            let obj_center = obj.center();

            if obj_center[axis] < threshold {
                bounds_a.grow(obj);
                a_count += 1;
            } else {
                bounds_b.grow(obj);
                b_count += 1;
            }
        }

        // discourage empty nodes
        if a_count == 0 || b_count == 0 {
            //log::info!("Invalid split, axis: {}, threshold: {}")
            return f32::MAX;
        }

        let a_cost = bounds_a.surface_area() * a_count as f32 * Self::OBJECT_COST;
        let b_cost = bounds_b.surface_area() * b_count as f32 * Self::OBJECT_COST;

        Self::NODE_COST + a_cost + b_cost
    }

    // returns (cost, axis, threshold)
    fn choose_split_axis<T: AsBoundingVolume + Clone + Sync>(
        bounds: BoundingVolume,
        list: &[T],
    ) -> (f32, usize, f32) {
        // compute the results for all 3 axes in parallel, and then choose the best at the end
        let results_per_axis: Vec<_> = (0..3)
            .into_par_iter()
            .map(|axis| {
                // if there are fewer objects in the volume, take a more accurate search
                let (bounds_min, bounds_max) = if list.len() < 10 {
                    let mut min = f32::INFINITY;
                    let mut max = f32::NEG_INFINITY;

                    // find min and max positions of the objects along this axis
                    for object in list {
                        let object_bounds = object.bounding_volume();

                        if object_bounds.min[axis] < min {
                            min = object_bounds.min[axis];
                        }
                        if object_bounds.max[axis] > max {
                            max = object_bounds.max[axis];
                        }
                    }

                    (min, max)
                } else {
                    (bounds.min[axis], bounds.max[axis])
                };

                let step_count = list.len().clamp(5, 20);
                let bounds_step = (bounds_max - bounds_min) / step_count as f32;

                // Vec<(cost, threshold)>
                // compute all the results in parallel and then choose the best one at the end
                let results: Vec<(f32, f32)> = (0..step_count)
                    .into_par_iter()
                    .map(|i| {
                        let threshold = bounds_min + bounds_step * (i as f32 + 0.5);
                        let cost = Self::evaluate_split_cost(list, axis, threshold);

                        (cost, threshold)
                    })
                    .collect();

                let mut best_cost = f32::INFINITY;
                let mut best_threshold = 0.0;

                for (cost, threshold) in results {
                    if cost < best_cost {
                        best_cost = cost;
                        best_threshold = threshold;
                    }
                }

                (best_cost, axis, best_threshold)
            })
            .collect();

        let mut best_cost = f32::INFINITY;
        let mut best_axis = 0;
        let mut best_threshold = 0.0;

        for (cost, axis, threshold) in results_per_axis {
            if cost < best_cost {
                best_cost = cost;
                best_axis = axis;
                best_threshold = threshold;
            }
        }

        (best_cost, best_axis, best_threshold)
    }

    pub fn split<T: AsBoundingVolume + Clone + Sync>(
        &mut self,
        list: &mut [T],
        nodes: &mut Vec<Self>,
        depth: u32,
        max_depth: u32,
    ) {
        if depth == max_depth || self.len <= 3 {
            return;
        }

        // the child containing objects greater than the split threshold
        let mut child_gt = Self {
            bounds: BoundingVolume::EMPTY,
            start_index: self.start_index,
            len: 0,
            child_node: 0,
        };

        // the child containing objects less than the split threshold
        let mut child_lt = Self {
            bounds: BoundingVolume::EMPTY,
            start_index: self.start_index,
            len: 0,
            child_node: 0,
        };

        let (cost, split_axis, split_threshold) =
            Self::choose_split_axis(self.bounds, self.slice(list));

        // don't split if the cost of the split would be greater than the current cost
        if cost >= self.cost() {
            return;
        }

        let greater = |object: &T| object.center()[split_axis] > split_threshold;

        for global_index in self.start_index..(self.start_index + self.len) {
            let global_index = global_index as usize;
            let object = &list[global_index];

            if greater(object) {
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

        if child_gt.len > 0 && child_lt.len > 0 {
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
}

pub struct BoundingVolumeHierarchy {
    pub version: u32,
    nodes: Vec<BvhNode>,
}

impl BoundingVolumeHierarchy {
    pub fn new<T: AsBoundingVolume + Clone + Sync>(list: &mut [T], version: u32) -> Self {
        if list.is_empty() {
            return Self {
                version,
                nodes: Vec::with_capacity(1),
            };
        }

        let instant = std::time::Instant::now();

        let max_depth = f32::log2(list.len() as f32) as u32 + 2;

        // create the root node
        let mut root = BvhNode::root(list);

        let initial_node_capacity = list.len() * 2 / 3;
        let mut nodes = Vec::with_capacity(initial_node_capacity);
        nodes.push(root);

        if !list.is_empty() {
            root.split(list, &mut nodes, 0, max_depth);
            nodes[0] = root;
        }

        let construction_time = instant.elapsed().as_secs_f64();

        let leaf_node_count = nodes.iter().filter(|node| node.child_node == 0).count();

        let min_leaf_object_count = nodes[1..]
            .iter()
            .filter(|node| node.child_node == 0)
            .map(|node| node.len)
            .min()
            .unwrap();

        let max_leaf_object_count = nodes[1..]
            .iter()
            .filter(|node| node.child_node == 0)
            .map(|node| node.len)
            .max()
            .unwrap();

        let average_leaf_object_count = nodes[1..]
            .iter()
            .filter(|node| node.child_node == 0)
            .map(|node| node.len)
            .sum::<u32>() as f32
            / leaf_node_count as f32;

        log::info!(
            r#"
            ---------- Bounding Volume Hierarchy Info ----------
            - Object count: {},
            - Number of nodes: {},
            - Node max depth: {},

            Leaf nodes:
                - Count: {}
                - Object count
                    - Min: {}
                    - Max: {}
                    - Average: {}

            Construction time: {} seconds
            ----------------------------------------------------
            "#,
            list.len(),
            nodes.len(),
            max_depth,
            leaf_node_count,
            min_leaf_object_count,
            max_leaf_object_count,
            average_leaf_object_count,
            construction_time
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
