use glam::Vec3;
use gpu_bytes::{AsStd140, AsStd430};
use gpu_bytes_derive::{AsStd140, AsStd430};
use rand::Rng;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum MaterialType {
    #[default]
    Lambertian = 0,
    Metal = 1,
    Dielectric = 2,
    Volume = 3,
}

impl AsStd140 for MaterialType {
    fn as_std140(&self) -> gpu_bytes::Std140Bytes {
        (*self as u32).as_std140()
    }
}

impl AsStd430 for MaterialType {
    fn as_std430(&self) -> gpu_bytes::Std430Bytes {
        (*self as u32).as_std430()
    }
}

#[derive(AsStd140, AsStd430, Debug, Clone, Copy)]
pub struct Material {
    pub albedo: Vec3,
    pub ty: MaterialType,
    pub emission: Vec3,
    pub roughness: f32,
    pub ior: f32,
    pub g: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::ONE,
            ty: MaterialType::Lambertian,
            emission: Vec3::ZERO,
            roughness: 0.0,
            ior: 0.0,
            g: 0.0,
        }
    }
}

impl Material {
    pub fn lambertian(albedo: Vec3) -> Self {
        Self {
            albedo,
            ty: MaterialType::Lambertian,
            ..Default::default()
        }
    }

    pub fn metal(albedo: Vec3, roughness: f32) -> Self {
        Self {
            albedo,
            ty: MaterialType::Metal,
            roughness,
            ..Default::default()
        }
    }

    pub fn dielectric(albedo: Vec3, roughness: f32, ior: f32) -> Self {
        Self {
            albedo,
            ty: MaterialType::Dielectric,
            roughness,
            ior,
            ..Default::default()
        }
    }

    pub fn with_emission(self, emission: Vec3) -> Self {
        Self { emission, ..self }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            ty: match rng.gen_range(0..3) {
                0 => MaterialType::Lambertian,
                1 => MaterialType::Metal,
                2 => MaterialType::Dielectric,
                _ => unreachable!(),
            },
            albedo: Vec3::new(
                rng.gen::<f32>().powf(2.2),
                rng.gen::<f32>().powf(2.2),
                rng.gen::<f32>().powf(2.2),
            ),
            emission: match rng.gen_bool(0.1) {
                // less emission is more common
                true => Vec3::new(
                    rng.gen_range(1.0f32..10.0),
                    rng.gen_range(1.0f32..10.0),
                    rng.gen_range(1.0f32..10.0),
                ),
                false => Vec3::ZERO,
            },
            roughness: rng.gen_range(0.0f32..1.0).powi(3),
            ior: rng.gen_range(0.5f32..3.0f32).powf(0.5),
            g: 0.0,
        }
    }
}
