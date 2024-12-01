use glam::UVec3;
use gpu_bytes::Std430Bytes;

use super::binding::Binding;

pub struct ComputePass<'a> {
    pub name: &'a str,
    pub workgroups: UVec3,
    pub pipeline: &'a wgpu::ComputePipeline,
    pub bindings: &'a [&'a Binding],
    pub push_constants: Option<Std430Bytes>,
}

impl<'a> ComputePass<'a> {
    pub fn draw(self, encoder: &mut wgpu::CommandEncoder) {
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

pub struct RenderPass<'a> {
    pub name: &'a str,
    pub color_attachments: &'a [Option<&'a wgpu::TextureView>],
    pub pipeline: &'a wgpu::RenderPipeline,
    pub bindings: &'a [&'a Binding],
    pub push_constants: Option<Vec<(wgpu::ShaderStages, Std430Bytes)>>,
}

impl<'a> RenderPass<'a> {
    pub fn draw(self, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(self.name),
            color_attachments: &self
                .color_attachments
                .iter()
                .map(|&view| {
                    Some(wgpu::RenderPassColorAttachment {
                        view: view?,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })
                })
                .collect::<Vec<_>>(),
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(self.pipeline);

        for (group, binding) in self.bindings.iter().enumerate() {
            render_pass.set_bind_group(group as u32, binding.bind_group(), &[]);
        }

        if let Some(push_constants) = self.push_constants {
            for (stage, data) in push_constants {
                render_pass.set_push_constants(stage, 0, data.as_slice());
            }
        }

        render_pass.draw(0..6, 0..1);
    }
}
