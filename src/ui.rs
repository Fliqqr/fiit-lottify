use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use bevy_vello::vello::kurbo::PathEl;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

use crate::{Exporter, FrameStepper};

pub fn controls_ui(
    exporter: Res<Exporter>,
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut fs: ResMut<FrameStepper>,
) {
    let control_panel =
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "Controls").resizable(false);

    control_panel.show(contexts.ctx_mut(), |ui| {
        // ui.heading("Control panel");

        ui.horizontal(|ui| {
            if ui.button("Export SVG").clicked() {
                println!("Export SVG");

                let mut document = Document::new().set("viewBox", (0, 0, 1000, 1000));

                if let Some(shapes) = &fs.shapes_buffer {
                    for mesh in shapes {
                        let path = Path::new().set("fill", "red");
                        let mut data = Data::new();

                        for p in &mesh.shape.paths {
                            if let PathEl::MoveTo(point) = p {
                                data = data.clone().move_to((point.x, point.y));
                            } else if let Some(point) = p.end_point() {
                                data = data.clone().line_by((point.x, point.y));
                            }
                        }

                        document = document.clone().add(path.set("d", data.close()));
                    }
                }

                svg::save("image.svg", &document).unwrap();
            }
            if ui.button("Export Lottie").clicked() {
                commands.run_system(exporter.lottie);
            }
        });
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
