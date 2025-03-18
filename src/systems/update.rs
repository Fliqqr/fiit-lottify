use bevy::prelude::*;
use bevy_vello::{
    vello::{
        kurbo::{Affine, Stroke},
        peniko,
    },
    VelloScene,
};

use super::cache::CachedMeshData;
use crate::{draw::generate_collection, FrameStepper, PathHighlight};

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut scene: Query<&mut VelloScene>,
    mut fs: ResMut<FrameStepper>,
    mut highlight: ResMut<PathHighlight>,
    projection: Query<&OrthographicProjection>,
    mesh_data: Res<CachedMeshData>,
) {
    let mut scene = scene.get_single_mut().expect("No VelloScene!");
    *scene = VelloScene::default();

    let shapes = if fs.is_animation_playing {
        highlight.paths.clear();

        // Generate shapes on every update
        generate_collection(
            mesh_data.ids.clone(),
            mesh_data.meshes.clone(),
            mesh_data.colors.clone(),
        )
    } else if fs.last_rendered_frame != fs.current_frame || fs.shapes_buffer.is_none() {
        highlight.paths.clear();

        let shapes = generate_collection(
            mesh_data.ids.clone(),
            mesh_data.meshes.clone(),
            mesh_data.colors.clone(),
        );

        fs.shapes_buffer = Some(shapes.clone());
        fs.last_rendered_frame = fs.current_frame;

        shapes
    } else {
        fs.shapes_buffer.as_ref().unwrap().clone()
    };

    if shapes.is_empty() {
        return;
    }

    for index in &mesh_data.ordering {
        let mesh = &shapes[*index];
        // }

        // for mesh in shapes {
        let color = mesh.color.to_linear();

        scene.fill(
            peniko::Fill::NonZero,
            Affine::IDENTITY,
            peniko::Color::rgb(color.red.into(), color.green.into(), color.blue.into()),
            None,
            &mesh.shape.paths.as_slice(),
        );
    }

    if !highlight.paths.is_empty() {
        // println!("highlight: {:?}", highlight.paths);

        let scale = projection.single().scale;

        scene.stroke(
            &Stroke::new(scale as f64),
            Affine::IDENTITY,
            peniko::Color::rgb(1.0, 1.0, 1.0),
            None,
            &highlight.paths.as_slice(),
        );
    }
}
