//! This example demonstrates how to use a storage buffer with `AsBindGroup` in a custom material.
use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{AsBindGroup, BufferUsages, ShaderRef},
        storage::ShaderStorageBuffer,
    },
    scene::SceneInstanceReady,
};

const SHADER_ASSET_PATH: &str = "shader.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PositionsShader {
    #[storage(0)]
    buffer: Handle<ShaderStorageBuffer>,
    pub positions: Arc<Mutex<Vec<[f32; 4]>>>,
}

impl Material for PositionsShader {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

fn create_shader(
    commands: &mut Commands,
    buffers: &mut ResMut<Assets<ShaderStorageBuffer>>,
    vertex_count: usize,
) -> PositionsShader {
    let mut buffer = ShaderStorageBuffer::from(vec![[f32::default(); 4]; vertex_count]);
    buffer.buffer_description.usage |= BufferUsages::COPY_SRC;

    let buffer = buffers.add(buffer);
    let positions = Arc::new(Mutex::new(Vec::new()));

    commands.spawn(Readback::buffer(buffer.clone())).observe({
        let positions = positions.clone();

        move |trigger: Trigger<ReadbackComplete>| {
            let data: Vec<[f32; 4]> = trigger.event().to_shader_type();
            let mut lock = positions.lock().unwrap();
            // info!("Buffer {:?}", data);

            *lock = data;
        }
    });

    PositionsShader { buffer, positions }
}

pub fn change_material(
    _trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut materials: ResMut<Assets<PositionsShader>>,
    meshes: Res<Assets<Mesh>>,
    query: Query<(Entity, &Mesh3d)>,
) {
    for (entity, mesh) in query.iter() {
        println!("Overriding material for: {:?}", mesh);

        let mesh = meshes.get(mesh).unwrap();

        let shader = create_shader(&mut commands, &mut buffers, mesh.count_vertices());
        let handle = materials.add(shader);

        commands
            .entity(entity)
            .insert(MeshMaterial3d(handle.clone()));
    }
}
