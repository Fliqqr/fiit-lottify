use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemId},
    prelude::*,
};
use bevy_vello::vello::kurbo::PathEl;

use esvg::page::Page;
use esvg::{create_document, Element};

use crate::{
    lottie::Lottie,
    systems::cache::CachedMeshData,
    vectorize::{generate_shapes, models::MeshShape},
    FrameStepper, FRAMES, FRAME_RATE, GLB,
};

#[derive(ScheduleLabel, Eq, PartialEq, Clone, Debug, Hash)]
pub struct ExportSchedule;

#[derive(Resource)]
pub struct ExportLottie {
    file: Lottie,
    frame: u64,
    last_mesh_data: CachedMeshData,
    pub exporting: bool,
    pub shape_frames: [Vec<MeshShape>; FRAMES as usize],
}

impl ExportLottie {
    pub fn new() -> Self {
        Self {
            file: Lottie::new(0),
            frame: 0,
            last_mesh_data: CachedMeshData {
                meshes: Vec::new(),
                ordering: Vec::new(),
            },
            exporting: false,
            shape_frames: [const { Vec::new() }; FRAMES as usize],
        }
    }
}

#[derive(Resource)]
pub struct Exporter {
    pub svg: SystemId,
    pub lottie: SystemId,
}

impl FromWorld for Exporter {
    fn from_world(world: &mut World) -> Self {
        Exporter {
            svg: world.register_system(export_svg),
            lottie: world.register_system(export_lottie),
        }
    }
}

fn export_lottie(mut export: ResMut<ExportLottie>, mut fs: ResMut<FrameStepper>) {
    export.file = Lottie::new(FRAME_RATE);
    export.exporting = true;
    export.last_mesh_data = CachedMeshData {
        meshes: Vec::new(),
        ordering: Vec::new(),
    };
    export.frame = 0;
    fs.current_frame = 0;
    // let mut file = Lottie::new(FRAME_RATE);

    // for (index, shapes) in export.shape_frames.iter().enumerate() {
    //     file.add_layer(
    //         shapes.clone(),
    //         &format!("frame{}", index),
    //         index as u64,
    //         (index + 1) as u64,
    //     );
    // }

    // file.save_as(&format!("{}.json", GLB));

    // 1. Save the current frame if it differs from the last

    //
}

pub fn export_system(
    mesh_data: Res<CachedMeshData>,
    animation_players: Query<&mut AnimationPlayer>,
    mut export: ResMut<ExportLottie>,
    mut fs: ResMut<FrameStepper>,
) {
    if !export.exporting {
        return;
    }

    update_animation(animation_players, &fs);

    if export.last_mesh_data == *mesh_data {
        return;
    }

    let shapes = generate_shapes(&mesh_data);
    export.last_mesh_data = mesh_data.clone();

    let frame = export.frame;
    println!("Frame: {}", frame);
    export
        .file
        .add_layer(shapes, &format!("frame{}", frame), frame, frame + 1);

    if fs.current_frame == FRAMES {
        println!("Saved: {}", &format!("{}.json", GLB));
        export.file.save_as(&format!("{}.json", GLB));
        export.exporting = false;
    }

    fs.current_frame += 1;
    export.frame = fs.current_frame;
}

// // Helper function to set the animation to the current_frame stored in FrameStepper
fn update_animation(mut animation_players: Query<&mut AnimationPlayer>, fs: &ResMut<FrameStepper>) {
    // println!("Update animation");
    for mut player in &mut animation_players {
        let Some((&index, _)) = player.playing_animations().next() else {
            continue;
        };
        let animation = player.animation_mut(index).unwrap();

        if !animation.is_paused() {
            animation.pause();
        }

        // println!("seek: {}", fs.current_frame);
        animation.seek_to(fs.current_frame as f32 / FRAME_RATE as f32);
        println!("Seek time: {}", animation.seek_time());
    }
}

#[allow(clippy::type_complexity)]
fn get_shapes(mesh_data: CachedMeshData) -> Vec<MeshShape> {
    // let _ = world.run_system_once(cache_mesh_data);
    // let mesh_data = world.get_resource::<CachedMeshData>().unwrap();

    let mut out = Vec::new();
    let shapes = generate_shapes(&mesh_data);
    for index in &mesh_data.ordering {
        out.push(shapes[*index].clone());
    }
    out
}

pub fn export_svg(world: &mut World) {
    let fs = world.get_resource::<FrameStepper>().unwrap();
    let cache = world.get_resource::<CachedMeshData>().unwrap();

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

    esvg::save(&format!("{}.svg", GLB), &doc).unwrap();
}
