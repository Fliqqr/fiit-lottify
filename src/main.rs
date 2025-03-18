use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_vello::{prelude::*, vello::kurbo::PathEl, VelloPlugin};

use export::Exporter;
use systems::{animation::Animations, cache::CachedMeshData};

use vello::AaConfig;

mod draw;
mod export;
mod lottie;
mod systems;
mod ui;

use draw::MeshShape;

/*
https://github.com/zimond/lottie-rs/
https://lottie.github.io/lottie-spec/1.0/single-page/
*/

/*
TODO:

1. Rotate scene to align camera with the viewport - DONE

2. GUI

3. Preview and framestepping - STARTED

4. Exporting frame as SVG

5. Mesh occlusion flitering

6. Shape ordering based on vertex positions
*/

#[derive(Component)]
struct GltfScene;

#[derive(Resource, Default)]
struct FrameStepper {
    current_frame: u64,
    total_frames: u64,
    last_rendered_frame: u64,
    is_animation_playing: bool,
    shapes_buffer: Option<Vec<MeshShape>>,
}

#[derive(Resource, Default)]
struct PathHighlight {
    paths: Vec<PathEl>,
}

impl FrameStepper {
    fn back(&mut self) {
        if self.current_frame > 0 {
            self.current_frame -= 1;
        }
    }

    fn forward(&mut self) {
        if self.current_frame < self.total_frames {
            self.current_frame += 1;
        }
    }
}

// const ASSETS: &str = "./assets";
const GLB: &str = "camera3.glb";

const FRAME_RATE: u64 = 60;
const FRAMES: u64 = 180;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.spawn((
        SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(GLB)),
            ..default()
        },
        GltfScene,
    ));

    commands.spawn(VelloSceneBundle::default());

    // Build the animation graph
    let mut graph = AnimationGraph::new();

    let animations = graph
        // Get the duration from the animation clip
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset(GLB)]
                .into_iter()
                .map(|path| asset_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });

    commands.insert_resource(FrameStepper {
        current_frame: 0,
        total_frames: FRAMES,
        last_rendered_frame: 0,
        is_animation_playing: false,
        shapes_buffer: None,
        // highlight: Vec::new(),
    });

    commands.insert_resource(PathHighlight { paths: Vec::new() });
}

#[derive(Resource)]
struct GltfHandleResource {
    gltf_handle: Handle<Gltf>,
}

fn assign_scene_handle(
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_handle_res: Option<Res<GltfHandleResource>>,
) {
    if let Some(gltf_handle_res) = gltf_handle_res {
        if let Some(gltf) = gltf_assets.get(&gltf_handle_res.gltf_handle) {
            let scene_handle = gltf.scenes[0].clone(); // Get first scene in the GLTF

            // Assign the scene to the previously spawned entity
            commands.spawn((
                SceneBundle {
                    scene: scene_handle,
                    ..default()
                },
                GltfScene,
            ));

            // Remove the temporary resource
            commands.remove_resource::<GltfHandleResource>();
        }
    }
}

#[allow(clippy::type_complexity)]
fn mesh_ordering(
    query: Query<&Handle<Mesh>, (With<Handle<Mesh>>, Added<Handle<Mesh>>)>,
    mut mesh_data: ResMut<CachedMeshData>,
) {
    if !query.is_empty() {
        println!("Mesh ordering");
        mesh_data.ordering = query.iter().enumerate().map(|(i, _)| i).collect();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_plugins(VelloPlugin {
            antialiasing: AaConfig::Msaa16,
            ..Default::default()
        })
        .add_systems(Startup, (setup,))
        .add_systems(
            Update,
            (
                mesh_ordering,
                assign_scene_handle,
                systems::camera::camera_setup,
                systems::cache::cache_mesh_data,
                systems::cache::cache_mesh_data.after(systems::camera::camera_setup),
                systems::animation::play_animation,
                systems::animation::keyboard_control,
                // systems::animation::get_animations,
                systems::update::update,
                ui::controls_ui,
            ),
        )
        .init_resource::<Exporter>()
        .init_resource::<CachedMeshData>()
        .init_resource::<Animations>()
        .insert_resource(FrameStepper {
            current_frame: 0,
            total_frames: FRAMES,
            last_rendered_frame: 0,
            is_animation_playing: false,
            shapes_buffer: None,
        })
        .insert_resource(PathHighlight { paths: Vec::new() })
        .run();
}
