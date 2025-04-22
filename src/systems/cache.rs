use bevy::prelude::*;

use crate::shader::{PositionsShader, VertexPositions};

#[derive(Clone)]
pub struct LottifyMesh {
    pub id: AssetId<Mesh>,
    pub mesh: Mesh,
    pub color: Color,
    pub positions: Vec<[f32; 4]>,
}

impl PartialEq for LottifyMesh {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.color == other.color && self.positions == other.positions
    }
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

#[derive(Resource, Default, PartialEq, Clone)]
pub struct CachedMeshData {
    pub meshes: Vec<LottifyMesh>,
    pub ordering: Vec<usize>,
    pub gpu_update: bool,
}

impl CachedMeshData {
    pub fn poll_update(&mut self) -> bool {
        if self.gpu_update {
            self.gpu_update = false;
            return true;
        }
        false
    }
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

        if vert_positions.poll_update(&materials) {
            mesh_data.gpu_update = true;
        }

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
    // println!("Cached");
}
