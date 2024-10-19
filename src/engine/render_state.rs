use std::sync::Arc;

use winit::window::Window;

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::FLOAT32_FILTERABLE
                        | wgpu::Features::RG11B10UFLOAT_RENDERABLE
                        | wgpu::Features::TEXTURE_BINDING_ARRAY
                        | wgpu::Features::PUSH_CONSTANTS
                        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER
                        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO,
                    required_limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::STORAGE_BINDING,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn reconfigure(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.reconfigure();
        }
    }

    pub fn begin_frame(
        &self,
    ) -> Result<(wgpu::CommandEncoder, wgpu::SurfaceTexture), wgpu::SurfaceError> {
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        let surface_texture = self.surface.get_current_texture()?;

        Ok((encoder, surface_texture))
    }

    pub fn finish_frame(
        &self,
        encoder: wgpu::CommandEncoder,
        surface_texture: wgpu::SurfaceTexture,
    ) {
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}
