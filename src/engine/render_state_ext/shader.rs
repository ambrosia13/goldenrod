use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::{engine::render_state::GpuState, util};

pub enum WgslShaderSource {
    File {
        name: String,
        source: String,
        path: PathBuf,
    },
    Fallback {
        name: String,
        path: PathBuf,
    },
}

impl WgslShaderSource {
    fn read_source<P: AsRef<Path>>(relative_path: P) -> Result<Self, std::io::Error> {
        let parent_path = std::env::current_dir()?;
        let path = parent_path.join(relative_path);

        let source = std::fs::read_to_string(&path)?;
        let source = util::preprocess::resolve_includes(source, &parent_path)?;

        let name = util::path_name_to_string(&path);

        Ok(Self::File { name, source, path })
    }

    pub fn load<P: AsRef<Path> + std::fmt::Debug>(relative_path: P) -> Self {
        match Self::read_source(&relative_path) {
            Ok(s) => s,
            Err(_) => {
                log::error!(
                    "Shader at path {:?} failed to load, substituting fallback shader.",
                    relative_path
                );
                Self::Fallback {
                    name: util::path_name_to_string(relative_path.as_ref()),
                    path: PathBuf::from(relative_path.as_ref()),
                }
            }
        }
    }

    pub fn reload(&mut self) {
        let path = self.path();
        *self = Self::load(path);
    }

    pub fn fallback<P: AsRef<Path> + std::fmt::Debug>(relative_path: P) -> Self {
        Self::Fallback {
            name: util::path_name_to_string(relative_path.as_ref()),
            path: PathBuf::from(relative_path.as_ref()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            WgslShaderSource::File { name, .. } => name,
            WgslShaderSource::Fallback { .. } => "fallback.wgsl",
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            WgslShaderSource::File { path, .. } => path,
            WgslShaderSource::Fallback { path, .. } => path,
        }
    }

    pub fn desc(&self) -> wgpu::ShaderModuleDescriptor {
        match self {
            WgslShaderSource::File {
                name,
                source,
                path: _,
            } => wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            },
            WgslShaderSource::Fallback { .. } => wgpu::ShaderModuleDescriptor {
                label: Some("Fallback Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "assets/fallback.wgsl"
                ))),
            },
        }
    }

    pub fn is_fallback(&self) -> bool {
        matches!(self, WgslShaderSource::Fallback { .. })
    }
}

pub struct WgpuShader {
    pub(in crate::engine::render_state_ext) source: WgslShaderSource,
    pub(in crate::engine::render_state_ext) module: wgpu::ShaderModule,

    pub(in crate::engine::render_state_ext) gpu: GpuState,
}

impl WgpuShader {
    pub fn source(&self) -> &WgslShaderSource {
        &self.source
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn recreate(&mut self) {
        self.source.reload();

        // so we can catch shader compilation errors instead of panicking
        self.gpu
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        self.module = self.gpu.device.create_shader_module(self.source.desc());

        let err = pollster::block_on(self.gpu.device.pop_error_scope());

        if err.is_some() {
            self.source = WgslShaderSource::fallback(self.source.path());
            self.module = self.gpu.device.create_shader_module(self.source.desc());
        }
    }
}

pub struct WgpuShaderProgram<'a> {
    pub vertex: Option<&'a WgpuShader>,
    pub fragment: Option<&'a WgpuShader>,
    pub compute: Option<&'a WgpuShader>,
}
