const STACK_SIZE = 12u;

struct Stack {
    len: u32,
    values: array<f32, STACK_SIZE>,
}

fn new_stack() -> Stack {
    return Stack(0, array<f32, STACK_SIZE>());
}

fn stack_is_empty(stack: ptr<function, Stack>) -> bool {
    return (*stack).len == 0u;
}

fn stack_is_full(stack: ptr<function, Stack>) -> bool {
    return (*stack).len >= STACK_SIZE;
}

fn push_to_stack(stack: ptr<function, Stack>, val: f32) {
    // only push if we still have capacity
    if !stack_is_full(stack) {
        (*stack).values[(*stack).len] = val;
    }

    (*stack).len += 1u;
}

fn pop_from_stack(stack: ptr<function, Stack>) {
    if !stack_is_empty(stack) {
        (*stack).len -= 1u;
    }

}

fn top_of_stack_or(stack: ptr<function, Stack>, or: f32) -> f32 {
    if stack_is_empty(stack) || ((*stack).len > STACK_SIZE) {
        return or;
    } else {
        return (*stack).values[(*stack).len - 1u];
    }
}