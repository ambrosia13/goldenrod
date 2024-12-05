use std::{
    borrow::Cow,
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::{engine::render_state::GpuState, util};

use super::RenderStateExt;

#[derive(Clone, Copy)]
pub enum ShaderBackend {
    Wgsl,
    Spirv,
}

pub struct ShaderMetadata {
    pub name: String,
    pub path: PathBuf,
    pub backend: ShaderBackend,
}

pub struct ShaderSourceNew {
    metadata: ShaderMetadata,
    source: Option<Vec<u8>>,
}

impl ShaderSourceNew {
    pub fn load_wgsl<P: AsRef<Path>>(path: P) -> Self {
        let name = util::path_name_to_string(&path);
        let path = path.as_ref().to_owned();

        let metadata = ShaderMetadata {
            name,
            path,
            backend: ShaderBackend::Wgsl,
        };

        fn read_shader_source<U: AsRef<Path>>(path: U) -> std::io::Result<Vec<u8>> {
            let parent_path = std::env::current_dir()?;
            let path = parent_path.join(path);

            let source = std::fs::read_to_string(&path)?;
            let source = util::preprocess::resolve_includes(source, &parent_path)?;

            Ok(source.into_bytes())
        }

        let source = read_shader_source(&metadata.path).ok();

        Self { metadata, source }
    }

    pub fn load_slang() -> Self {
        unimplemented!()
    }

    pub fn load_spirv() -> Self {
        unimplemented!()
    }

    pub fn reload(&mut self) {
        let path = &self.metadata.path;

        match self.metadata.backend {
            ShaderBackend::Wgsl => *self = Self::load_wgsl(path),
            ShaderBackend::Spirv => unimplemented!(),
        }
    }

    pub fn make_fallback(&mut self) {
        self.source = None;
    }

    pub fn is_fallback(&self) -> bool {
        self.source.is_none()
    }

    pub fn backend(&self) -> ShaderBackend {
        self.metadata.backend
    }

    pub fn source_str(&self) -> Option<&str> {
        match self.backend() {
            ShaderBackend::Wgsl => Some(std::str::from_utf8(self.source.as_ref()?).unwrap()),
            ShaderBackend::Spirv => panic!("Can't get source strings for binary Spir-V format"),
        }
    }

    pub fn source_bytes(&self) -> Option<&[u8]> {
        unimplemented!()
    }

    pub fn descriptor(&self) -> wgpu::ShaderModuleDescriptor<'_> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: match self.backend() {
                ShaderBackend::Wgsl => match self.source_str() {
                    Some(source) => wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
                    None => wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                        "assets/fallback.wgsl"
                    ))),
                },
                ShaderBackend::Spirv => match self.source_bytes() {
                    Some(bytes) => wgpu::ShaderSource::SpirV(wgpu::util::make_spirv_raw(bytes)),
                    None => todo!(),
                },
            },
        }
    }
}

pub struct ShaderNew {
    source: ShaderSourceNew,
    module: wgpu::ShaderModule,

    gpu_state: GpuState,
}

impl ShaderNew {
    pub fn new(gpu_state: &impl RenderStateExt, mut source: ShaderSourceNew) -> Self {
        gpu_state
            .device()
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let mut module = gpu_state.device().create_shader_module(source.descriptor());

        let compile_error = pollster::block_on(gpu_state.device().pop_error_scope());

        if compile_error.is_some() {
            source.make_fallback();
            module = gpu_state.device().create_shader_module(source.descriptor());
        }

        Self {
            source,
            module,
            gpu_state: gpu_state.as_gpu_state(),
        }
    }

    pub fn source(&self) -> &ShaderSourceNew {
        &self.source
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn recreate(&mut self) {
        self.source.reload();

        self.gpu_state
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        self.module = self
            .gpu_state
            .device
            .create_shader_module(self.source.descriptor());

        let err = pollster::block_on(self.gpu_state.device.pop_error_scope());

        if err.is_some() {
            self.source.make_fallback();
            self.module = self
                .gpu_state
                .device
                .create_shader_module(self.source.descriptor());
        }
    }
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
