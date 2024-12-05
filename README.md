# goldenrod renderering engine

TODO: new updated screenshots

![glass ball](https://github.com/ambrosia13/forget_me_not/blob/main/screenshots/glass_ball.png?raw=true)

![metallic balls](https://github.com/ambrosia13/forget_me_not/blob/main/screenshots/metallic_balls.png?raw=true)

![mirror room](https://github.com/ambrosia13/forget_me_not/blob/main/screenshots/mirror_room.png?raw=true)

![glass boxes](https://github.com/ambrosia13/forget_me_not/blob/main/screenshots/glass_boxes.png?raw=true)

`goldenrod` is a rendering engine and path tracer written in Rust. It uses the `wgpu` library, which is a native Rust implementation of the WebGPU specification.

The current path tracing system can represent a few different kinds of materials, and uses spectral rendering to accurately simulate all wavelengths of light, not just R, G, and B wavelengths.

Four types of geometry are implemented:
- spheres
- planes
- axis-aligned bounding boxes (AABBs)
- triangles

For triangles, a Bounding-Volume-Hierarchy is constructed to accelerate intersection tests. This makes `goldenrod` capable of rendering models with millions of triangles at relatively fast speeds.

# notes

This project is licensed under the GNU General Public License v3.0.

Much of the code is adapted from my other big rendering project, [Forget-me-not Shaders](https://github.com/ambrosia13/ForgetMeNot-Shaders), which is a rasterization-based graphics overhaul for the game Minecraft, and it's written in GLSL.