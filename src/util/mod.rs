use std::path::Path;

pub mod gltf;
pub mod preprocess;

pub fn path_name_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}
