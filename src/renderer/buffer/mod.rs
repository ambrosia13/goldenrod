pub mod bvh;
pub mod object;
pub mod screen;

/// Runtime-size arrays in storage buffers will allocate at least this many elements to avoid allocating
/// buffers on the gpu with zero size.
pub const MIN_STORAGE_ARRAY_CAPACITY: usize = 1;
