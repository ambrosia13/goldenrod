#include assets/shaders/lib/header.wgsl

struct Vertex {
    position: vec2<f32>,
    uv: vec2<f32>,
    texcoord: vec2<f32>,
}

@group(0) @binding(0)
var<storage> vertices: array<Vertex>;

@group(0) @binding(1)
var<storage> indices: array<u32>;


@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let index = indices[vertex_index];
    let vertex = vertices[index];
    
    out.clip_position = vec4(vertex.position.xy, 0.0, 1.0);
    out.uv = vertex.uv;
    out.texcoord = vertex.texcoord;

    return out;
}