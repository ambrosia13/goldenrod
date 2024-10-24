// pub struct RenderPass {}

// impl RenderPass {
//     pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
//         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//             label: Some(&self.name),
//             ..Default::default()
//         });

//         render_pass.draw(vertices, instances);
//     }
// }

use glam::UVec3;
use gpu_bytes::Std430Bytes;

use super::{binding::WgpuBinding, shader::WgpuShader};

pub struct WgpuComputePass<'a> {
    pub name: &'a str,
    pub workgroups: UVec3,
    pub pipeline: &'a wgpu::ComputePipeline,
    pub bindings: &'a [&'a WgpuBinding],
    pub push_constants: Option<Std430Bytes>,
    pub shader: &'a WgpuShader,
}

impl<'a> WgpuComputePass<'a> {
    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(self.name),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(self.pipeline);

        for (group, binding) in self.bindings.iter().enumerate() {
            compute_pass.set_bind_group(group as u32, binding.bind_group(), &[]);
        }

        if let Some(push_constants) = &self.push_constants {
            compute_pass.set_push_constants(0, push_constants.as_slice());
        }

        compute_pass.dispatch_workgroups(self.workgroups.x, self.workgroups.y, self.workgroups.z);
    }
}
