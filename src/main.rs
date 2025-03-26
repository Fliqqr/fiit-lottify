use bevy::{
    app::MainSchedulePlugin,
    prelude::*,
    render::{Render, RenderApp, RenderPlugin},
};
use bevy_egui::EguiPlugin;
use bevy_vello::{prelude::*, vello::kurbo::PathEl, VelloPlugin};

use export::Exporter;
use export_schedule::SteppingPlugin;
use shader::PositionsShader;
use systems::{animation::Animations, cache::CachedMeshData};

use vello::AaConfig;

mod draw;
mod export;
mod export_schedule;
mod lottie;
mod shader;
mod systems;
mod ui;

use draw::MeshShape;

/*
https://github.com/zimond/lottie-rs/
https://lottie.github.io/lottie-spec/1.0/single-page/
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
const GLB: &str = "Fox_baked.glb";

const FRAME_RATE: u64 = 30;
const FRAMES: u64 = 30;

const PREVIEW_3D: bool = false;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let gltf_handle = asset_server.load(GLB);

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(GLB))),
        GltfScene,
    ));

    if !PREVIEW_3D {
        commands.spawn(VelloSceneBundle::default());
    }

    // Insert a placeholder for animations until the GLTF is fully loaded
    commands.insert_resource(PendingAnimations { gltf_handle });
}

// System to load animations dynamically when GLTF is ready
fn load_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_assets: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Option<Res<PendingAnimations>>,
) {
    if let Some(pending) = query {
        if let Some(gltf) = gltf_assets.get(&pending.gltf_handle.clone()) {
            println!("Loading animations: {}", gltf.animations.len());

            let mut graph = AnimationGraph::new();

            let animations = graph
                .add_clips(
                    (0..gltf.animations.len())
                        .map(|i| asset_server.load(GltfAssetLabel::Animation(i).from_asset(GLB)))
                        .collect::<Vec<_>>(),
                    1.0,
                    graph.root,
                )
                .collect();

            let graph_handle = graphs.add(graph);

            commands.insert_resource(Animations {
                animations,
                graph: graph_handle.clone(),
            });

            // Remove the temporary pending resource
            commands.remove_resource::<PendingAnimations>();
        }
    }
}

#[derive(Resource)] // Resource to track pending animations
struct PendingAnimations {
    gltf_handle: Handle<Gltf>,
}

#[allow(clippy::type_complexity)]
fn mesh_ordering(query: Query<&Mesh3d, Added<Mesh3d>>, mut mesh_data: ResMut<CachedMeshData>) {
    if !query.is_empty() {
        println!("Mesh ordering");
        mesh_data.ordering = query.iter().enumerate().map(|(i, _)| i).collect();
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, MaterialPlugin::<PositionsShader>::default()))
        .add_plugins(EguiPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_plugins(VelloPlugin {
            antialiasing: AaConfig::Msaa16,
            ..Default::default()
        })
        .add_plugins(
            SteppingPlugin::default().add_schedule(Update), // .add_schedule(Render),
        )
        .add_systems(Startup, (setup,))
        .add_systems(
            Update,
            (
                mesh_ordering,
                load_animations,
                systems::camera::camera_setup,
                systems::cache::cache_mesh_data,
                // systems::cache::cache_mesh_data.after(systems::camera::camera_setup),
                systems::animation::play_animation,
                systems::animation::keyboard_control,
                // systems::animation::get_animations,
                systems::update::update,
                ui::controls_ui,
            ),
        )
        .add_observer(shader::change_material)
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
        .insert_resource(PathHighlight { paths: Vec::new() });

    let render_app = app.get_sub_app_mut(RenderApp).expect("No render subapp");

    render_app.add_systems(Render, test);

    app.run();
}

fn test() {
    println!("Render");
}
