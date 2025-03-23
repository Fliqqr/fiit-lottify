use bevy::{prelude::*, utils::tracing};
use bevy_vello::{
    vello::{
        kurbo::{Affine, Rect, Shape, Stroke},
        peniko,
    },
    VelloScene,
};

use super::cache::CachedMeshData;
use crate::{
    draw::{generate_collection, generate_collection2},
    FrameStepper, PathHighlight,
};

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut scene: Query<&mut VelloScene>,
    mut fs: ResMut<FrameStepper>,
    mut highlight: ResMut<PathHighlight>,
    projection: Query<&OrthographicProjection>,
    mesh_data: Res<CachedMeshData>,
) {
    let Ok(mut scene) = scene.get_single_mut() else {
        tracing::error!("No Vello scene!");
        return;
    };
    *scene = VelloScene::default();

    // let shapes = if fs.is_animation_playing {
    //     highlight.paths.clear();

    //     // Generate shapes on every update
    //     generate_collection2(
    //         mesh_data.ids.clone(),
    //         mesh_data.meshes.clone(),
    //         mesh_data.colors.clone(),
    //         mesh_data.positions.clone(),
    //     )
    // } else if fs.last_rendered_frame != fs.current_frame || fs.shapes_buffer.is_none() {
    //     highlight.paths.clear();

    //     let shapes = generate_collection2(
    //         mesh_data.ids.clone(),
    //         mesh_data.meshes.clone(),
    //         mesh_data.colors.clone(),
    //         mesh_data.positions.clone(),
    //     );

    //     fs.shapes_buffer = Some(shapes.clone());
    //     fs.last_rendered_frame = fs.current_frame;

    //     shapes
    // } else {
    //     fs.shapes_buffer.as_ref().unwrap().clone()
    // };

    if mesh_data.positions.is_empty() {
        println!("Empty pos");
        return;
    }

    let shapes = generate_collection2(
        mesh_data.ids.clone(),
        mesh_data.meshes.clone(),
        mesh_data.colors.clone(),
        mesh_data.positions.clone(),
    );

    scene.fill(
        peniko::Fill::NonZero,
        Affine::IDENTITY,
        peniko::Color::from_rgb8(255, 0, 0),
        None,
        &Rect {
            x0: -1000.0,
            y0: -1000.0,
            x1: 1000.0,
            y1: 1000.0,
        },
    );

    if shapes.is_empty() {
        println!("Empty shapes");
        return;
    }

    for index in &mesh_data.ordering {
        let mesh = &shapes[*index];
        // }

        // println!("drawing: {:?}", shapes);

        // for mesh in shapes {
        let color = mesh.color.to_linear();

        scene.fill(
            peniko::Fill::NonZero,
            Affine::IDENTITY,
            // peniko::Color::WHITE,
            peniko::Color::new([color.red, color.blue, color.green, 1.0]),
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
            peniko::Color::BLACK,
            // peniko::Color::rgb(0.8, 0.85, 1.0),
            None,
            &highlight.paths.as_slice(),
        );
    }
}
