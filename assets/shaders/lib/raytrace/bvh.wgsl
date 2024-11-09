struct BoundingVolume {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct BvhNode {
    bounds: BoundingVolume,
    start_index: u32,
    len: u32,
    child_node: u32,
}

struct BvhHit {
    success: bool,
    distance: f32,
}

fn ray_bounding_volume_intersect(ray: Ray, bounding_volume: BoundingVolume) -> BvhHit {
    let t_min = (bounding_volume.min - ray.pos) / ray.dir;
    let t_max = (bounding_volume.max - ray.pos) / ray.dir;

    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);

    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);

    let front_face = t_near == 0.0;

    if front_face { // ray inside box
        return BvhHit(true, t_far);   
    } else {
        return BvhHit(t_far >= t_near && t_far > 0.0, t_near);
    }
}

const NODE_STACK_SIZE = 32u;

struct NodeStack {
    len: u32,
    values: array<u32, NODE_STACK_SIZE>,
}

fn new_node_stack() -> NodeStack {
    return NodeStack(0, array<u32, NODE_STACK_SIZE>());
}

fn node_stack_is_empty(node_stack: ptr<function, NodeStack>) -> bool {
    return (*node_stack).len == 0u;
}

fn node_stack_is_full(node_stack: ptr<function, NodeStack>) -> bool {
    return (*node_stack).len >= NODE_STACK_SIZE;
}

fn push_to_node_stack(node_stack: ptr<function, NodeStack>, val: u32) {
    // only push if we still have capacity
    if !node_stack_is_full(node_stack) {
        (*node_stack).values[(*node_stack).len] = val;
    }

    (*node_stack).len += 1u;
}

fn pop_from_node_stack(node_stack: ptr<function, NodeStack>) {
    if !node_stack_is_empty(node_stack) {
        (*node_stack).len -= 1u;
    }

}

fn top_of_node_stack_or(node_stack: ptr<function, NodeStack>, or: u32) -> u32 {
    if node_stack_is_empty(node_stack) || ((*node_stack).len > NODE_STACK_SIZE) {
        return or;
    } else {
        return (*node_stack).values[(*node_stack).len - 1u];
    }
}