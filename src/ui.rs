use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use bevy_vello::vello::kurbo::PathEl;
use esvg::page::Page;
use esvg::{create_document, Element};

use crate::{CachedMeshData, Exporter, FrameStepper, PathHighlight};

pub fn export_svg(fs: &FrameStepper, cache: &CachedMeshData) {
    println!("Export SVG");

    let page = Page::letter(100);
    let mut doc = create_document(&page);
    doc.set("viewBox", "-100, -100, 200, 200");

    if let Some(shapes) = &fs.shapes_buffer {
        for index in &cache.ordering {
            let mesh = &shapes[*index];
            // }

            // for (index, mesh) in shapes.iter().enumerate() {
            let mut group = Element::new("g");
            group.set("class", format!("Mesh{}", index));

            let mut points = Vec::new();

            for p in &mesh.shape.paths {
                match p {
                    PathEl::MoveTo(point) | PathEl::LineTo(point) => {
                        points.push(polygonical::point::Point::new(point.x, point.y))
                    }
                    PathEl::ClosePath => {
                        let mut path = esvg::path::create_closed(&points);
                        path.set("fill", mesh.color.to_srgba().to_hex());

                        group.add(&path);
                        points.clear();
                    }
                    _ => panic!("Unsupported pathel"),
                }
            }

            doc.add(&group);
        }
    }

    esvg::save("image.svg", &doc).unwrap();
}

pub fn controls_ui(
    exporter: Res<Exporter>,
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut fs: ResMut<FrameStepper>,
    mut highlight: ResMut<PathHighlight>,
    mut mesh_data: ResMut<CachedMeshData>,
) {
    let control_panel =
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "Controls").resizable(false);

    control_panel.show(contexts.ctx_mut(), |ui| {
        // ui.heading("Control panel");

        ui.horizontal(|ui| {
            if ui.button("Export SVG").clicked() {
                export_svg(&fs, &mesh_data);
            }
            if ui.button("Export Lottie").clicked() {
                commands.run_system(exporter.lottie);
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
