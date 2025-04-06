use draw::generate_shape;
use models::MeshShape;

use crate::systems::cache::CachedMeshData;

mod draw;
pub mod models;

pub fn round_to(num: impl Into<f64>, places: u32) -> f64 {
    let num = num.into();
    let pow = 10_f64.powf(places as f64);

    (num * pow).round() / pow
}

pub fn generate_shapes(mesh_data: &CachedMeshData) -> Vec<MeshShape> {
    // println!("Gen2: {:?}", positions);

    let mut out = Vec::new();

    for mesh in mesh_data.meshes.iter() {
        let mut tmp: Vec<[f32; 3]> = Vec::new();

        for pos in &mesh.positions {
            tmp.push([pos[0], pos[1], pos[2]]);
        }

        let indices = mesh.mesh.indices().unwrap().iter().collect::<Vec<usize>>();

        // Could run these in parallel
        if let Ok(shapes) = generate_shape(&indices, tmp, mesh.color, mesh.id) {
            out.push(shapes);
        }
    }

    out
}
