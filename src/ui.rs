use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Align2},
};
use nfd::Response;

use crate::{CachedMeshData, Exporter, FrameStepper, GltfScene, PathHighlight, load_file};

#[allow(clippy::too_many_arguments)]
pub fn controls_ui(
    exporter: Res<Exporter>,
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut fs: ResMut<FrameStepper>,
    mut highlight: ResMut<PathHighlight>,
    mut mesh_data: ResMut<CachedMeshData>,
    asset_server: Res<'_, AssetServer>,
    existing_scenes: Query<Entity, With<GltfScene>>,
) {
    let control_panel =
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "Controls").resizable(false);

    control_panel.show(contexts.ctx_mut(), |ui| {
        // ui.heading("Control panel");

        ui.horizontal(|ui| {
            if ui.button("Export SVG").clicked() {
                commands.run_system(exporter.svg);
            }

            if ui.button("Export Lottie").clicked() {
                println!("UI Export Lottie");
                commands.run_system(exporter.lottie);
            }

            if ui.button("Open File").clicked() {
                let result = nfd::open_file_dialog(None, None).unwrap_or_else(|e| {
                    panic!("{}", e);
                });

                match result {
                    Response::Okay(file_path) => {
                        load_file(file_path.clone(), commands, asset_server, existing_scenes);

                        println!("File path = {:?}", file_path)
                    }
                    Response::OkayMultiple(files) => println!("Files {:?}", files),
                    Response::Cancel => println!("User canceled"),
                }
            }
        });
    });

    let shape_layers = egui::SidePanel::new(egui::panel::Side::Left, "Layers").resizable(true);

    shape_layers.show(contexts.ctx_mut(), |ui| {
        let mut new_ordering = Vec::new();

        if let Some(shapes) = &fs.shapes_buffer {
            new_ordering = mesh_data.ordering.clone();

            if shapes.is_empty() {
                return;
            }

            for (index, shape_idx) in mesh_data.ordering.iter().enumerate() {
                let shape = &shapes[*shape_idx];
                // }

                // for (shape_idx, shape) in shapes.iter().enumerate() {
                let id = match shape.mesh_id {
                    AssetId::Index { index, marker: _ } => format!("{}", index.to_bits()),
                    AssetId::Uuid { uuid } => uuid.to_string(),
                };

                let name = shape.name.clone().unwrap_or(format!("Shape {}", id));

                // ui.dnd_drag_source(id, payload, add_contents)

                ui.vertical(|ui| {
                    if ui.button("up").clicked() {
                        new_ordering.swap(index, index - 1);
                    };
                    if ui.button("down").clicked() {
                        new_ordering.swap(index, index + 1);
                    };

                    if ui
                        .heading(name)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        highlight.paths = shape.shape.paths.clone();
                    };

                    let mut index = 0;
                    let mut path_buffer = Vec::new();

                    for path in shape.shape.paths.iter() {
                        path_buffer.push(*path);

                        if path.end_point().is_none() {
                            if ui
                                .label(format!("Path {}", index))
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                println!("clicked path {}", index);
                                path_buffer.push(*path);

                                highlight.paths = path_buffer.clone();
                            };
                            index += 1;

                            path_buffer.clear();
                        }
                    }
                });

                ui.separator();
            }
        }
        if !new_ordering.is_empty() {
            // println!("{:?}", new_ordering);

            mesh_data.ordering = new_ordering;
        }
    });

    let animation_bar = egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "Animation");

    animation_bar.show(contexts.ctx_mut(), |ui| {
        // ui.heading("Animation control");

        let frames = fs.total_frames;

        ui.horizontal(|ui| {
            if ui.button("<").clicked() {
                fs.back();
                fs.shapes_buffer = None;
            }

            if ui.button(">").clicked() {
                fs.forward();
                fs.shapes_buffer = None;
            }
        });

        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = ui.available_width() * 0.95;
            ui.add(
                egui::Slider::new(&mut fs.current_frame, 0..=frames)
                    .step_by(1.0)
                    .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.25 }),
            )
        });
    });

    let stats = egui::Window::new("Stats")
        .collapsible(true)
        .resizable(false)
        .anchor(Align2::RIGHT_TOP, [-5.0, 30.0]);

    stats.show(contexts.ctx_mut(), |ui| {
        ui.label("Path segments: ");
        ui.label("Shapes: ");
    });
}
