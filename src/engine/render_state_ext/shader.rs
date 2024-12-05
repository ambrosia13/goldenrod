use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::{engine::render_state::GpuState, util};

#[derive(Clone, Copy)]
pub enum ShaderBackend {
    Wgsl,
    Spirv,
}

pub struct ShaderFile {
    name: String,
    path: PathBuf,
    backend: ShaderBackend,
}

pub struct ShaderSourceNew {
    file: ShaderFile,
    source: Option<Vec<u8>>,
}

impl ShaderSourceNew {
    pub fn load_wgsl() -> Self {
        todo!()
    }

    pub fn load_slang() -> Self {
        todo!()
    }

    pub fn load_spirv() -> Self {
        todo!()
    }

    pub fn reload(&mut self) {
        todo!()
    }

    pub fn is_fallback(&self) -> bool {
        self.source.is_none()
    }

    pub fn backend(&self) -> ShaderBackend {
        self.file.backend
    }

    pub fn source_str(&self) -> &str {
        todo!()
    }

    pub fn source_bytes(&self) -> &[u8] {
        todo!()
    }

    pub fn descriptor(&self) -> wgpu::ShaderModuleDescriptor<'_> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: match self.backend() {
                ShaderBackend::Wgsl => wgpu::ShaderSource::Wgsl(Cow::Borrowed(self.source_str())),
                ShaderBackend::Spirv => {
                    wgpu::ShaderSource::SpirV(wgpu::util::make_spirv_raw(self.source_bytes()))
                }
            },
        }
    }
}

pub struct ShaderNew {
    source: ShaderSourceNew,
    module: wgpu::ShaderModule,

    gpu_state: GpuState,
}

pub enum ShaderSource {
    File {
        name: String,
        source: String,
        path: PathBuf,
        backend: ShaderBackend,
    },
    Fallback {
        path: PathBuf,
        backend: ShaderBackend,
    },
}

impl ShaderSource {
    fn read_source<P: AsRef<Path>>(
        relative_path: P,
        backend: ShaderBackend,
    ) -> Result<Self, std::io::Error> {
        let parent_path = std::env::current_dir()?;
        let path = parent_path.join(relative_path);

        let source = std::fs::read_to_string(&path)?;
        let source = util::preprocess::resolve_includes(source, &parent_path)?;

        let name = util::path_name_to_string(&path);

        // match &*path
        //     .extension()
        //     .expect("Shader source files should have an extension")
        //     .to_string_lossy()
        // {
        //     "wgsl" => {}
        //     "spirv" => {}
        //     _ => todo!(),
        // };

        Ok(Self::File {
            name,
            source,
            path,
            backend,
        })
    }

    pub fn load<P: AsRef<Path> + std::fmt::Debug>(
        relative_path: P,
        backend: ShaderBackend,
    ) -> Self {
        match Self::read_source(&relative_path, backend) {
            Ok(s) => s,
            Err(_) => {
                log::error!(
                    "Shader at path {:?} failed to load, substituting fallback shader.",
                    relative_path
                );
                Self::Fallback {
                    path: PathBuf::from(relative_path.as_ref()),
                    backend,
                }
            }
        }
    }

    pub fn reload(&mut self) {
        let path = self.path();
        *self = Self::load(path, self.backend());
    }

    pub fn fallback<P: AsRef<Path> + std::fmt::Debug>(relative_path: P) -> Self {
        Self::Fallback {
            path: PathBuf::from(relative_path.as_ref()),
            backend: ShaderBackend::Wgsl,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ShaderSource::File { name, .. } => name,
            ShaderSource::Fallback { .. } => "fallback.wgsl",
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            ShaderSource::File { path, .. } => path,
            ShaderSource::Fallback { path, .. } => path,
        }
    }

    pub fn source(&self) -> &str {
        match self {
            ShaderSource::File { source, .. } => source,
            ShaderSource::Fallback { .. } => include_str!("assets/fallback.wgsl"),
        }
    }

    pub fn backend(&self) -> ShaderBackend {
        match self {
            ShaderSource::File { backend, .. } => *backend,
            ShaderSource::Fallback { backend, .. } => *backend,
        }
    }

    pub fn desc(&self) -> wgpu::ShaderModuleDescriptor {
        // let source = match self.backend() {
        //     ShaderBackend::Wgsl => wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
        //     ShaderBackend::Spirv => todo!(),
        // };

        match self {
            ShaderSource::File { name, source, .. } => wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            },
            ShaderSource::Fallback { .. } => wgpu::ShaderModuleDescriptor {
                label: Some("Fallback Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "assets/fallback.wgsl"
                ))),
            },
        }
    }

    pub fn is_fallback(&self) -> bool {
        matches!(self, ShaderSource::Fallback { .. })
    }
}

pub struct Shader {
    pub(in crate::engine::render_state_ext) source: ShaderSource,
    pub(in crate::engine::render_state_ext) module: wgpu::ShaderModule,

    pub(in crate::engine::render_state_ext) gpu_state: GpuState,
}

impl Shader {
    pub fn source(&self) -> &ShaderSource {
        &self.source
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn recreate(&mut self) {
        self.source.reload();

        // so we can catch shader compilation errors instead of panicking
        self.gpu_state
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        self.module = self
            .gpu_state
            .device
            .create_shader_module(self.source.desc());

        let err = pollster::block_on(self.gpu_state.device.pop_error_scope());

        if err.is_some() {
            self.source = ShaderSource::fallback(self.source.path());
            self.module = self
                .gpu_state
                .device
                .create_shader_module(self.source.desc());
        }
    }
}
