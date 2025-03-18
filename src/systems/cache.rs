use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CachedMeshData {
    pub ids: Vec<AssetId<Mesh>>,
    pub meshes: Vec<Mesh>,
    pub colors: Vec<Color>,
    pub ordering: Vec<usize>,
}

// TODO: This only needs to run every time the frame changes
// PS: Caching only makes sense if the scene rotation is static
#[allow(clippy::complexity)]
pub fn cache_mesh_data(
    query: Query<(&GlobalTransform, &Handle<Mesh>, &Handle<StandardMaterial>), With<Handle<Mesh>>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut mesh_data: ResMut<CachedMeshData>,
) {
    // println!("Caching mesh data...");

    let mut ids = Vec::new();
    let mut t_meshes = Vec::new();
    let mut t_colors = Vec::new();

    for (global_transform, mesh_handle, material_handle) in query.iter() {
        let material = materials
            .get(material_handle)
            .expect("Mesh has no material");

        if let Some(mesh) = meshes.get(mesh_handle) {
            ids.push(mesh_handle.id());

            let transformed_mesh = mesh.clone().transformed_by((*global_transform).into());

            t_meshes.push(transformed_mesh);
            t_colors.push(material.base_color);
        }
    }

    if t_meshes.is_empty() {
        // println!("No meshes");
        return;
    }

    // println!("DONE");
    mesh_data.ids = ids;
    mesh_data.meshes = t_meshes;
    mesh_data.colors = t_colors;
}
