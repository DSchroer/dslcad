use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

const SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6538896235495937986);

pub struct PointRenderPlugin;

impl Plugin for PointRenderPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../shaders/point_shader.wgsl")),
        );
        app.add_plugin(MaterialPlugin::<PointMaterial>::default());
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "14130c7f-3f82-46d3-89f1-47b3007785a3"]
pub struct PointMaterial {
    #[uniform(0)]
    pub color: Color,
}

impl Material for PointMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER_HANDLE.typed())
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER_HANDLE.typed())
    }
}
