use bevy::prelude::*;

use crate::shader::{PositionsShader, VertexPositions};

#[derive(Clone)]
pub struct LottifyMesh {
    pub id: AssetId<Mesh>,
    pub mesh: Mesh,
    pub color: Color,
    pub positions: Vec<[f32; 4]>,
}

impl LottifyMesh {
    pub fn new(id: AssetId<Mesh>, mesh: Mesh, color: Color, positions: Vec<[f32; 4]>) -> Self {
        Self {
            id,
            mesh,
            color,
            positions,
        }
    }
}

#[derive(Resource, Default)]
pub struct CachedMeshData {
    pub meshes: Vec<LottifyMesh>,
    pub ordering: Vec<usize>,
}

// TODO: This only needs to run every time the frame changes
// PS: Caching only makes sense if the scene rotation is static
#[allow(clippy::complexity)]
pub fn cache_mesh_data(
    query: Query<(&GlobalTransform, &Mesh3d, &VertexPositions)>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<PositionsShader>>,
    mut mesh_data: ResMut<CachedMeshData>,
) {
    // println!("Caching mesh data...");
    let mut data = Vec::new();

    for (global_transform, mesh_handle, vert_positions) in query.iter() {
        let pos = vert_positions.get_positions(&materials);

        if let Some(mesh) = meshes.get(mesh_handle) {
            let transformed_mesh = mesh.clone().transformed_by((*global_transform).into());

            data.push(LottifyMesh::new(
                mesh_handle.id(),
                transformed_mesh,
                vert_positions.color,
                pos,
            ));
        }
    }

    if data.is_empty() {
        // println!("No meshes");
        return;
    }

    mesh_data.meshes = data;
    println!("Cached");
}
