#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_bindings::mesh
#import bevy_render::maths::affine3_to_square

struct LineSettings {
    color: vec4<f32>,
    width: f32,
    depth_bias: f32,
};

@group(2) @binding(0) var<uniform> settings: LineSettings;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) other: vec3<f32>,
    @location(2) side: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

fn clip_near_plane(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    // Move a if a is behind the near plane and b is in front.
    if a.z > a.w && b.z <= b.w {
        let distance_a = a.z - a.w;
        let distance_b = b.z - b.w;
        let t = distance_a / (distance_a - distance_b);
        return a + (b - a) * t;
    }
    return a;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let model = affine3_to_square(mesh[vertex.instance_index].world_from_local);

    var clip0 = view.clip_from_world * model * vec4(vertex.position, 1.0);
    var clip1 = view.clip_from_world * model * vec4(vertex.other, 1.0);

    // Manual near plane clipping to avoid errors when doing the perspective
    // divide inside this shader.
    clip0 = clip_near_plane(clip0, clip1);
    clip1 = clip_near_plane(clip1, clip0);

    let resolution = vec2(view.viewport.z, view.viewport.w);
    let screen0 = resolution * (0.5 * clip0.xy / clip0.w + 0.5);
    let screen1 = resolution * (0.5 * clip1.xy / clip1.w + 0.5);

    let x_basis = normalize(screen1 - screen0);
    let y_basis = vec2(-x_basis.y, x_basis.x);

    let pt = screen0 + 0.5 * settings.width * vertex.side * y_basis;

    var depth: f32 = clip0.z;
    if settings.depth_bias >= 0.0 {
        depth = depth * (1.0 - settings.depth_bias);
    } else {
        let epsilon = 4.88e-04;
        depth = depth * exp2(-settings.depth_bias * log2(clip0.w / depth - epsilon));
    }

    return VertexOutput(
        vec4(clip0.w * ((2.0 * pt) / resolution - 1.0), depth, clip0.w),
        settings.color,
    );
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
