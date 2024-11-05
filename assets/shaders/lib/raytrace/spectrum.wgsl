const WAVELENGTH_TO_XYZ_START = 360.0;
const WAVELENGTH_TO_XYZ_END = 360.0 + 471.0;
const WAVELENGTH_TO_XYZ_STEP = 1.0;

const RGB_TO_SPECTRAL_INTENSITY_START = 380.0;
const RGB_TO_SPECTRAL_INTENSITY_END = 780.0;
const RGB_TO_SPECTRAL_INTENSITY_STEP = 5.0;

const WAVELENGTH_RANGE_START = 380.0;
const WAVELENGTH_RANGE_END = 780.0;

fn generate_wavelength() -> f32 {
    return next_f32() * (WAVELENGTH_RANGE_END - WAVELENGTH_RANGE_START) + WAVELENGTH_RANGE_START;
}

fn wavelength_to_xyz(lut: texture_storage_1d<rgba32float, read>, wavelength: f32) -> vec3<f32> {
    let translated = clamp(
        (wavelength - WAVELENGTH_TO_XYZ_START), 
        0.0, 
        WAVELENGTH_TO_XYZ_END - WAVELENGTH_TO_XYZ_START
    ) * WAVELENGTH_TO_XYZ_STEP;

    let icoord = u32(translated);
    let fcoord = fract(translated);

    let xyz = mix(textureLoad(lut, icoord).xyz, textureLoad(lut, icoord + 1).xyz, fcoord);

    return xyz;
}

fn rgb_to_spectral_intensity(lut: texture_storage_1d<rgba32float, read>, rgb: vec3<f32>, wavelength: f32) -> f32 {
    let translated = clamp(
        (wavelength - RGB_TO_SPECTRAL_INTENSITY_START), 
        0.0, 
        RGB_TO_SPECTRAL_INTENSITY_END - RGB_TO_SPECTRAL_INTENSITY_START
    ) / RGB_TO_SPECTRAL_INTENSITY_STEP;

    let icoord = u32(translated);
    let fcoord = fract(translated);

    let color = mix(textureLoad(lut, icoord).xyz, textureLoad(lut, icoord + 1).xyz, fcoord);
    let intensity = dot(rgb, color);

    return intensity;
}

fn xyz_to_rgb(xyz: vec3<f32>) -> vec3<f32> {
    let xyz_to_rgb_matrix = mat3x3(
        3.2404542,-0.9692660, 0.0556434,
        -1.5371385, 1.8760108,-0.2040259,
        -0.4985314, 0.0415560, 1.0572252
    );

    return xyz_to_rgb_matrix * xyz; //pow(max(vec3(0.0), xyz_to_rgb_matrix * xyz), vec3(2.2));
}

// fn basis_r(wavelength: f32) -> f32 {
//     let center = 650.0; // Peak for red
//     let width = 27.0;
//     return exp(-pow(wavelength - center, 2.0) / (2.0 * width * width));
// }

// fn basis_g(wavelength: f32) -> f32 {
//     let center = 550.0; // Peak for green
//     let width = 13.0;
//     return exp(-pow(wavelength - center, 2.0) / (2.0 * width * width));
// }

// fn basis_b(wavelength: f32) -> f32 {
//     let center = 440.0; // Peak for blue
//     let width = 15.0;
//     return exp(-pow(wavelength - center, 2.0) / (2.0 * width * width));
// }

// fn basis_r(wavelength: f32) -> f32 {
//     if wavelength >= 610.0 && wavelength <= 750.0 {
//         return 1.0;
//     } else if wavelength >= 570.0 && wavelength < 610.0 {
//         return (wavelength - 570.0) / (610.0 - 570.0);
//     }
//     return 0.0;
// }

// fn basis_g(wavelength: f32) -> f32 {
//     if wavelength >= 495.0 && wavelength < 570.0 {
//         return 1.0;
//     } else if wavelength >= 450.0 && wavelength < 495.0 {
//         return (wavelength - 450.0) / (495.0 - 450.0);
//     } else if wavelength >= 570.0 && wavelength < 610.0 {
//         return (610.0 - wavelength) / (610.0 - 570.0);
//     }
//     return 0.0;
// }

// fn basis_b(wavelength: f32) -> f32 {
//     if wavelength >= 380.0 && wavelength < 450.0 {
//         return 1.0;
//     } else if wavelength >= 450.0 && wavelength < 495.0 {
//         return (495.0 - wavelength) / (495.0 - 450.0);
//     }
//     return 0.0;
// }

// fn rgb_to_spectral_intensity(rgb: vec3<f32>, wavelength: f32) -> f32 {
//     return basis_r(wavelength) * rgb.r + basis_g(wavelength) * rgb.g + basis_b(wavelength) * rgb.b;
// }
