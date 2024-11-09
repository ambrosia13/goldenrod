use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    path::{Path, PathBuf},
};

use glam::{Quat, Vec3};
use gltf::{mesh::Mode, Gltf};

use crate::state::{material::Material, object::Triangle};

#[derive(Debug)]
pub enum GltfLoadError {
    InvalidFileStructure,
    IoError(std::io::Error),
    GltfError(gltf::Error),
}

impl From<std::io::Error> for GltfLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<gltf::Error> for GltfLoadError {
    fn from(value: gltf::Error) -> Self {
        Self::GltfError(value)
    }
}

impl Display for GltfLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for GltfLoadError {}

pub fn load_triangles_from_gltf<P: AsRef<Path>>(
    relative_path: P,
    offset: Vec3,
    rotation: Quat,
    scale: f32,
    material: Material,
) -> Result<Vec<Triangle>, GltfLoadError> {
    let parent_path = std::env::current_dir()?;
    let path = parent_path.join(relative_path);

    let paths = std::fs::read_dir(&path)?;
    let paths: Vec<PathBuf> = paths
        .into_iter()
        .map(|r| r.map(|p| p.path()))
        .collect::<Result<Vec<_>, _>>()?;

    let gltf_path = paths
        .into_iter()
        .find(|p| p.extension().unwrap().to_str().unwrap() == "gltf")
        .ok_or(GltfLoadError::InvalidFileStructure)?;

    let gltf = Gltf::open(gltf_path)?;

    let bin_data = gltf.blob.as_deref();

    let mut uri_data: HashMap<&str, Vec<u8>> = HashMap::new();

    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {}
            gltf::buffer::Source::Uri(uri) => {
                let data = std::fs::read(path.join(uri))?;
                uri_data.insert(uri, data);
            }
        }
    }

    let mut triangles = Vec::new();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                continue;
            }

            let reader = primitive.reader(|buf| match buf.source() {
                gltf::buffer::Source::Bin => bin_data,
                gltf::buffer::Source::Uri(uri) => Some(uri_data.get(&uri).unwrap()),
            });

            if let (Some(positions), Some(indices)) =
                (reader.read_positions(), reader.read_indices())
            {
                let positions: Vec<[f32; 3]> = positions.collect();

                for chunk in indices.into_u32().collect::<Vec<_>>().chunks(3) {
                    triangles.push(Triangle::new(
                        (rotation * (Vec3::from(positions[chunk[0] as usize]) * scale)) + offset,
                        (rotation * (Vec3::from(positions[chunk[1] as usize]) * scale)) + offset,
                        (rotation * (Vec3::from(positions[chunk[2] as usize]) * scale)) + offset,
                        material,
                    ));
                }
            }
        }
    }

    Ok(triangles)
}

pub fn load_triangles_from_glb<P: AsRef<Path>>(
    relative_path: P,
    offset: Vec3,
    rotation: Quat,
    scale: f32,
    material: Material,
) -> Result<Vec<Triangle>, GltfLoadError> {
    let parent_path = std::env::current_dir()?;
    let path = parent_path.join(relative_path);

    if path
        .extension()
        .ok_or(GltfLoadError::InvalidFileStructure)?
        .to_string_lossy()
        != "glb"
    {
        return Err(GltfLoadError::InvalidFileStructure);
    }

    let gltf = Gltf::open(path)?;

    let bin_data = gltf.blob.as_deref();

    let mut triangles = Vec::new();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                continue;
            }

            let reader = primitive.reader(|buf| match buf.source() {
                gltf::buffer::Source::Bin => bin_data,
                gltf::buffer::Source::Uri(_) => None,
            });

            if let (Some(positions), Some(indices)) =
                (reader.read_positions(), reader.read_indices())
            {
                let positions: Vec<[f32; 3]> = positions.collect();

                for chunk in indices.into_u32().collect::<Vec<_>>().chunks(3) {
                    triangles.push(Triangle::new(
                        (rotation * (Vec3::from(positions[chunk[0] as usize]) * scale)) + offset,
                        (rotation * (Vec3::from(positions[chunk[1] as usize]) * scale)) + offset,
                        (rotation * (Vec3::from(positions[chunk[2] as usize]) * scale)) + offset,
                        material,
                    ));
                }
            }
        }
    }

    Ok(triangles)
}
