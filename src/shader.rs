//! This example demonstrates how to use a storage buffer with `AsBindGroup` in a custom material.
use std::sync::{Arc, Mutex};

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexAttributeValues},
        render_resource::{
            AsBindGroup, BufferUsages, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
        storage::ShaderStorageBuffer,
    },
    scene::SceneInstanceReady,
};
use bevy_vello::vello::wgpu::VertexFormat;

const SHADER_ASSET_PATH: &str = "shader.wgsl";

// Do we need this?
const ATTRIBUTE_MORPH_TARGET: MeshVertexAttribute = MeshVertexAttribute::new(
    "Morph_Target",
    123,
    bevy::render::render_resource::VertexFormat::Float32x3,
);

const ATTRIBUTE_MESH_OFFSET: MeshVertexAttribute =
    MeshVertexAttribute::new("MeshOffset", 988540917, VertexFormat::Uint32);

#[derive(Component)]
pub struct VertexPositions {
    handle: Handle<PositionsShader>,
    offset: usize,
    len: usize,
    pub color: Color,
}

impl VertexPositions {
    pub fn get_positions(&self, materials: &Res<Assets<PositionsShader>>) -> Vec<[f32; 4]> {
        let shader = materials.get(&self.handle).unwrap();
        let lock = shader.positions.lock().unwrap();

        (*lock)[self.offset..(self.offset + self.len)].to_vec()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PositionsShader {
    #[storage(0)]
    buffer: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only)]
    mesh_offsets: Handle<ShaderStorageBuffer>,
    pub positions: Arc<Mutex<Vec<[f32; 4]>>>,
}

impl Material for PositionsShader {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Append the new attribute to the existing layout
        // descriptor.vertex.buffers.push(layout.0.layout().clone());

        let mut vertex_attributes = Vec::new();

        if layout.0.contains(Mesh::ATTRIBUTE_POSITION) {
            vertex_attributes.push(Mesh::ATTRIBUTE_POSITION.at_shader_location(0));
        }
        if layout.0.contains(Mesh::ATTRIBUTE_NORMAL) {
            vertex_attributes.push(Mesh::ATTRIBUTE_NORMAL.at_shader_location(1));
        }
        if layout.0.contains(Mesh::ATTRIBUTE_UV_0) {
            vertex_attributes.push(Mesh::ATTRIBUTE_UV_0.at_shader_location(2));
        }
        if layout.0.contains(Mesh::ATTRIBUTE_UV_1) {
            vertex_attributes.push(Mesh::ATTRIBUTE_UV_1.at_shader_location(3));
        }
        if layout.0.contains(Mesh::ATTRIBUTE_TANGENT) {
            vertex_attributes.push(Mesh::ATTRIBUTE_TANGENT.at_shader_location(4));
        }
        if layout.0.contains(Mesh::ATTRIBUTE_COLOR) {
            vertex_attributes.push(Mesh::ATTRIBUTE_COLOR.at_shader_location(5));
        }
        if layout.0.contains(Mesh::ATTRIBUTE_JOINT_INDEX) {
            vertex_attributes.push(Mesh::ATTRIBUTE_JOINT_INDEX.at_shader_location(6));
        }
        if layout.0.contains(Mesh::ATTRIBUTE_JOINT_WEIGHT) {
            vertex_attributes.push(Mesh::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(7));
        }

        vertex_attributes.push(ATTRIBUTE_MESH_OFFSET.at_shader_location(20));

        let layout = layout.0.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![layout];

        Ok(())
    }
}

pub fn change_material(
    _trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut materials: ResMut<Assets<PositionsShader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    standard_materials: Res<Assets<StandardMaterial>>,
    query: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>)>,
) {
    let mut total_vertices = 0;
    let mut mesh_offsets = Vec::new();

    for (_, mesh, _) in query.iter() {
        let mesh = meshes.get(mesh).unwrap();

        mesh_offsets.push(total_vertices as u32);
        total_vertices += mesh.count_vertices();
    }

    println!("total vertices: {}", total_vertices);
    println!("offsets: {:?}", mesh_offsets);

    let mut buffer = ShaderStorageBuffer::from(vec![[f32::default(); 4]; total_vertices]);
    buffer.buffer_description.usage |= BufferUsages::COPY_SRC;

    let buffer = buffers.add(buffer);
    let positions = Arc::new(Mutex::new(vec![[f32::default(); 4]; total_vertices]));

    commands.spawn(Readback::buffer(buffer.clone())).observe({
        let positions = positions.clone();

        move |trigger: Trigger<ReadbackComplete>| {
            // println!("readback complete");
            let data: Vec<[f32; 4]> = trigger.event().to_shader_type();

            {
                let mut lock = positions.lock().unwrap();
                // println!("buffer len: {}", data.len());
                // info!("Buffer {:?}", data);

                // if *lock == data {
                //     println!("Same buffer");
                // }

                *lock = data;
            }
        }
    });

    let shader = PositionsShader {
        buffer,
        mesh_offsets: buffers.add(ShaderStorageBuffer::from(mesh_offsets.clone())),
        positions,
    };

    let handle = materials.add(shader);

    for ((entity, mesh_handle, material_handle), offset) in query.iter().zip(mesh_offsets) {
        let mesh = meshes.get(mesh_handle).unwrap();
        let num_vertices = mesh.count_vertices();

        println!("Overriding material for: {:?}", mesh_handle);
        println!("offset: {} len: {}", offset, num_vertices);

        // Clone the mesh to modify it
        let mut new_mesh = mesh.clone();

        // // Add a zeroed-out morph target attribute
        let blank_morph_target = vec![[0.0, 0.0, 0.0]; num_vertices];
        new_mesh.insert_attribute(
            ATTRIBUTE_MORPH_TARGET,
            VertexAttributeValues::Float32x3(blank_morph_target),
        );
        // Possibly hacky af
        new_mesh.insert_attribute(
            ATTRIBUTE_MESH_OFFSET,
            VertexAttributeValues::Uint32(vec![offset; new_mesh.count_vertices()]),
        );

        let original_material = standard_materials.get(material_handle).unwrap();
        println!("Original color: {:?}", original_material.base_color);

        // Insert modified mesh
        commands.entity(entity).insert((
            MeshMaterial3d(handle.clone()),
            VertexPositions {
                handle: handle.clone(),
                offset: offset as usize,
                len: num_vertices,
                color: original_material.base_color,
            },
        ));

        // Update the asset
        meshes.insert(mesh_handle.clone(), new_mesh);
    }
}
