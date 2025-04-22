use bevy::{prelude::*, utils::tracing};
use bevy_vello::{
    vello::{
        kurbo::{Affine, Stroke},
        peniko,
    },
    VelloScene,
};

use super::cache::CachedMeshData;
use crate::{
    export::ExportLottie, vectorize::generate_shapes, FrameStepper, PathHighlight, FRAMES,
};

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut scene: Query<&mut VelloScene>,
    mut fs: ResMut<FrameStepper>,
    mut highlight: ResMut<PathHighlight>,
    projection: Query<&OrthographicProjection>,
    mut mesh_data: ResMut<CachedMeshData>,
    mut export: ResMut<ExportLottie>,
) {
    let Ok(mut scene) = scene.get_single_mut() else {
        tracing::error!("No Vello scene!");
        return;
    };
    *scene = VelloScene::default();

    let shapes = if fs.is_animation_playing {
        highlight.paths.clear();

        // Generate shapes on every update
        generate_shapes(&mesh_data)
    // This lags behind the actual frame by 1 for some reason
    } else if mesh_data.poll_update() || fs.shapes_buffer.is_none() {
        highlight.paths.clear();

        let shapes = generate_shapes(&mesh_data);

        fs.shapes_buffer = Some(shapes.clone());

        if fs.current_frame < FRAMES {
            if shapes.is_empty() {
                return;
            }
            export.shape_frames[fs.current_frame as usize] = shapes.clone();
        }

        fs.updated = false;

        shapes
    } else {
        // println!("YEP");
        fs.shapes_buffer.as_ref().unwrap().clone()
    };

    if shapes.is_empty() {
        return;
    }

    for index in &mesh_data.ordering {
        if index >= &shapes.len() {
            break;
        }

        let mesh = &shapes[*index];

        // for mesh in shapes {
        let color = mesh.color.to_linear();

        scene.fill(
            peniko::Fill::NonZero,
            Affine::IDENTITY,
            // peniko::Color::WHITE,
            peniko::Color::new([color.red, color.green, color.blue, 1.0]),
            None,
            &mesh.shape.paths.as_slice(),
        );
    }

    if !highlight.paths.is_empty() {
        // println!("highlight: {:?}", highlight.paths);

        let scale = projection.single().scale * 2.0;

        scene.stroke(
            &Stroke::new(scale as f64),
            Affine::IDENTITY,
            // peniko::Color::BLACK,
            // peniko::Color::from((0.8, 0.85, 1.0)),
            peniko::Color::from_rgb8(120, 180, 255),
            None,
            &highlight.paths.as_slice(),
        );
    }
}
