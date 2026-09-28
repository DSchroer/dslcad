// The `ShaderType` derive generates a `check` function that rustc reports as
// dead code (with the field spans), so dead code analysis is disabled here.
#![allow(dead_code)]

use bevy::asset::{load_internal_asset, RenderAssetUsages};
use bevy::mesh::{
    Indices, MeshVertexAttribute, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexFormat,
};
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use dslcad_storage::protocol::Point;

const LINE_SHADER_HANDLE: Handle<Shader> =
    bevy::asset::uuid_handle!("0000000000000000cd2f4c969a7d149b");

const ATTRIBUTE_OTHER: MeshVertexAttribute = MeshVertexAttribute::new(
    "Vertex_Other",
    14785120311401321628,
    VertexFormat::Float32x3,
);
const ATTRIBUTE_SIDE: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Side", 14785120311401321629, VertexFormat::Float32);

/// Renders every line of a part as a single batched mesh. Each line segment is
/// expanded into a quad in the vertex shader, so the whole part is drawn with
/// one draw call.
pub struct LineMaterialPlugin;

impl Plugin for LineMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, LINE_SHADER_HANDLE, "lines.wgsl", Shader::from_wgsl);

        app.add_plugins(MaterialPlugin::<LineMaterial>::default());
        app.add_plugins(MaterialPlugin::<AnnotationLineMaterial>::default());
    }
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct LineSettings {
    pub color: Vec4,
    pub border_color: Vec4,
    pub width: f32,
    pub depth_bias: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub settings: LineSettings,
}

impl LineMaterial {
    pub fn new(color: Color, width: f32) -> Self {
        let color = color.to_linear().to_vec4();
        Self {
            settings: LineSettings {
                color,
                border_color: color,
                width,
                depth_bias: 0.0,
            },
        }
    }

    pub fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.settings.depth_bias = depth_bias;
        self
    }
}

impl Material for LineMaterial {
    fn vertex_shader() -> ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.vertex.buffers = vec![layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_OTHER.at_shader_location(1),
            ATTRIBUTE_SIDE.at_shader_location(2),
        ])?];
        Ok(())
    }
}

/// A line material that ignores depth testing, so drawing annotations are
/// always visible on top of the model and background.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AnnotationLineMaterial {
    #[uniform(0)]
    pub settings: LineSettings,
}

impl AnnotationLineMaterial {
    pub fn new(color: Color, border: Color, width: f32) -> Self {
        Self {
            settings: LineSettings {
                color: color.to_linear().to_vec4(),
                border_color: border.to_linear().to_vec4(),
                width,
                depth_bias: 0.0,
            },
        }
    }
}

impl Material for AnnotationLineMaterial {
    fn vertex_shader() -> ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    /// Render annotations in the transparent pass so they are always drawn
    /// after the opaque parts and therefore always sit on top of the model.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.vertex.buffers = vec![layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_OTHER.at_shader_location(1),
            ATTRIBUTE_SIDE.at_shader_location(2),
        ])?];

        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_compare = CompareFunction::Always;
            depth.depth_write_enabled = false;
        }

        Ok(())
    }
}

/// Expand every line segment into a quad. Each vertex stores one endpoint, the
/// other endpoint and which side of the line it is on. The vertex shader uses
/// this to build a screen space quad with a constant pixel width.
pub fn lines_to_mesh(lines: &[Vec<Point>]) -> Mesh {
    let segment_count: usize = lines.iter().map(|line| line.len().saturating_sub(1)).sum();

    let mut positions = Vec::with_capacity(segment_count * 4);
    let mut others = Vec::with_capacity(segment_count * 4);
    let mut sides = Vec::with_capacity(segment_count * 4);
    let mut indices = Vec::with_capacity(segment_count * 6);

    for line in lines {
        for segment in line.windows(2) {
            let start = segment[0].map(|value| value as f32);
            let end = segment[1].map(|value| value as f32);
            if start == end {
                continue;
            }

            let base = positions.len() as u32;
            positions.extend([start, start, end, end]);
            others.extend([end, end, start, start]);
            sides.extend([-1.0, 1.0, -1.0, 1.0]);
            indices.extend([base, base + 1, base + 2, base + 2, base + 1, base + 3]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(ATTRIBUTE_OTHER, others);
    mesh.insert_attribute(ATTRIBUTE_SIDE, sides);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_expands_every_segment_into_a_quad() {
        let mesh = lines_to_mesh(&[vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]]]);

        assert_eq!(8, mesh.count_vertices());
        assert_eq!(12, mesh.indices().unwrap().len());
    }

    #[test]
    fn it_skips_degenerate_segments() {
        let mesh = lines_to_mesh(&[vec![[0., 0., 0.], [0., 0., 0.]]]);

        assert_eq!(0, mesh.count_vertices());
        assert_eq!(0, mesh.indices().unwrap().len());
    }
}
