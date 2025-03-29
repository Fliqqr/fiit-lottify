use bevy::prelude::*;

use crate::shader::{PositionsShader, VertexPositions};

#[derive(Resource, Default)]
pub struct CachedMeshData {
    pub ids: Vec<AssetId<Mesh>>,
    pub meshes: Vec<Mesh>,
    pub colors: Vec<Color>,
    pub ordering: Vec<usize>,
    pub positions: Vec<Vec<[f32; 4]>>,
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

    let mut ids = Vec::new();
    let mut t_meshes = Vec::new();
    let mut t_colors = Vec::new();
    let mut positions = Vec::new();

    for (global_transform, mesh_handle, vert_positions) in query.iter() {
        let pos = vert_positions.get_positions(&materials);
        // println!("pos");
        // println!("pos {}: {:?}", pos.len(), pos);
        positions.push(pos);

        if let Some(mesh) = meshes.get(mesh_handle) {
            ids.push(mesh_handle.id());

            let transformed_mesh = mesh.clone().transformed_by((*global_transform).into());

            t_meshes.push(transformed_mesh);
            // t_colors.push(Color::srgb_from_array([1.0, 0.0, 0.0]));
            t_colors.push(vert_positions.color);
        }
    }

    if t_meshes.is_empty() {
        // println!("No meshes");
        return;
    }

    if positions.is_empty() {
        return;
    }

    // println!("DONE");
    mesh_data.ids = ids;
    mesh_data.meshes = t_meshes;
    mesh_data.colors = t_colors;
    mesh_data.positions = positions;
}
