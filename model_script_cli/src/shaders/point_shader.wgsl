#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

struct CustomMaterial {
    color: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> material: CustomMaterial;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let w = (view.view_proj * mesh.model * vec4<f32>(0.0, 0.0, 0.0, 1.0)).w;

    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position * w * 0.005, 1.0));
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return material.color;
}