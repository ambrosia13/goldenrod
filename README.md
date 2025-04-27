# goldenrod renderering engine

![objects](https://github.com/ambrosia13/goldenrod/blob/main/screenshots/objects.png?raw=true)

![suzanne](https://github.com/ambrosia13/goldenrod/blob/main/screenshots/suzanne.png?raw=true)

![diamond dragon](https://github.com/ambrosia13/goldenrod/blob/main/screenshots/diamond_dragon.png?raw=true)

`goldenrod` is a rendering engine and path tracer written in Rust. It uses the `wgpu` library, which is a native Rust implementation of the WebGPU specification.

The current path tracing system can represent a few different kinds of materials, and uses spectral rendering to accurately simulate all wavelengths of light, not just red, green, and blue wavelengths. This makes it possible to display much more accurate colors, as well as simulate certain wavelength-dependent light physics, such as dispersion.

Other than path tracing, `goldenrod` implements cubemap loading, bloom, and the construction of an acceleration structure for triangles. It also uses real camera response functions to display colors exactly as real-life cameras would display them in photographs.

Four types of geometry are implemented:
- spheres
- planes
- axis-aligned bounding boxes (AABBs)
- triangles

For triangles, a Bounding-Volume-Hierarchy is constructed to accelerate intersection tests. This makes `goldenrod` capable of rendering models with millions of triangles at relatively fast speeds.

# notes

This project is licensed under the GNU General Public License v3.0.

Much of the code is adapted from my other big rendering project, [Forget-me-not Shaders](https://github.com/ambrosia13/ForgetMeNot-Shaders), which is a rasterization-based graphics overhaul for the game Minecraft, and it's written in GLSL.
