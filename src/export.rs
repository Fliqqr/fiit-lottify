use bevy::{
    ecs::system::{RunSystemOnce, SystemId},
    prelude::*,
};
use bevy_vello::vello::kurbo::PathEl;

use esvg::page::Page;
use esvg::{create_document, Element};

use crate::{
    draw::{generate_collection, MeshShape},
    lottie::Lottie,
    systems::cache::CachedMeshData,
    FrameStepper, FRAMES, FRAME_RATE, GLB,
};

#[derive(Resource)]
pub struct Exporter {
    pub lottie: SystemId,
    pub svg: SystemId,
}

impl FromWorld for Exporter {
    fn from_world(world: &mut World) -> Self {
        Exporter {
            lottie: world.register_system(export_lottie),
            svg: world.register_system(export_svg),
        }
    }
}

pub fn export_lottie(world: &mut World) {
    println!("Exporting lottie...");
    let mut file = Lottie::new(FRAME_RATE);

    for frame in 0..FRAMES {
        println!("Frame: {}/{}", frame, FRAMES);

        world.run_system_once_with(frame, update_frame);
        world.run_system_once(update_animation);
        let shapes = world.run_system_once(get_shapes).unwrap();

        // Needed for animation to update
        world.run_schedule(PreUpdate);
        world.run_schedule(PostUpdate);

        file.add_layer(shapes, &format!("Frame {}", frame), frame, frame + 1);
    }

    file.save_as(&format!("{}.json", GLB));
}

fn update_frame(In(frame): In<u64>, mut fs: ResMut<FrameStepper>) {
    fs.current_frame = frame;
}

// Helper function to set the animation to the current_frame stored in FrameStepper
fn update_animation(mut animation_players: Query<&mut AnimationPlayer>, fs: ResMut<FrameStepper>) {
    println!("Update animation");
    for mut player in &mut animation_players {
        let Some((&index, _)) = player.playing_animations().next() else {
            continue;
        };
        let animation = player.animation_mut(index).unwrap();

        if !animation.is_paused() {
            animation.pause();
        }

        println!("seek: {}", fs.current_frame);
        animation.seek_to(fs.current_frame as f32 / FRAME_RATE as f32);
    }
}

#[allow(clippy::type_complexity)]
fn get_shapes(mesh_data: Res<CachedMeshData>) -> Vec<MeshShape> {
    let mut out = Vec::new();
    let shapes = generate_collection(
        mesh_data.ids.clone(),
        mesh_data.meshes.clone(),
        mesh_data.colors.clone(),
    );
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
