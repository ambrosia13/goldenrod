use super::buffer::WgpuBuffer;

pub struct RenderTimestamps {
    set: wgpu::QuerySet,
    resolve_buffer: WgpuBuffer,
    destination_buffer: WgpuBuffer,
}
