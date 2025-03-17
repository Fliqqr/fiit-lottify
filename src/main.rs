use bevy::ecs::system::RunSystemOnce;
use bevy::scene::SceneInstance;
use bevy::{ecs::system::SystemId, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_pancam::DirectionKeys;
use bevy_vello::vello::kurbo::{PathEl, Stroke};
use bevy_vello::{prelude::*, VelloPlugin};

use std::time::Duration;

use kurbo::Affine;
use lottie::Lottie;
use vello::AaConfig;

mod draw;
use draw::{generate_collection, MeshShape};

mod lottie;

mod ui;

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

#[derive(Resource)]
struct Exporter {
    lottie: SystemId,
}

impl FromWorld for Exporter {
    fn from_world(world: &mut World) -> Self {
        Exporter {
            lottie: world.register_system(export_lottie),
        }
    }
}

#[derive(Component)]
struct GltfScene;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_plugins(VelloPlugin {
            antialiasing: AaConfig::Msaa16,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        // .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(
            Update,
            (
                camera_setup,
                cache_mesh_data,
                cache_mesh_data.after(camera_setup),
                mesh_ordering,
                play_animation,
                keyboard_control,
                ui::controls_ui,
                update,
            ),
        )
        .init_resource::<Exporter>()
        .init_resource::<CachedMeshData>()
        .run();
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

#[derive(Resource, Default)]
struct FrameStepper {
    current_frame: u64,
    total_frames: u64,
    last_rendered_frame: u64,
    is_animation_playing: bool,
    shapes_buffer: Option<Vec<MeshShape>>,
    // highlight: Vec<PathEl>,
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

// const GLB: &str = "ico.glb";
const GLB: &str = "camera3.glb";
const FRAME_RATE: u64 = 60;
const FRAMES: u64 = 60;

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

#[allow(clippy::type_complexity)]
fn camera_setup(
    mut commands: Commands,
    cam_query: Query<(Entity, &Camera, &GlobalTransform), (With<Camera3d>, Added<Camera3d>)>,
    mut scene_transform: Query<&mut Transform, (With<SceneInstance>, Without<Camera3d>)>,
) {
    if let Ok((entity, camera, cam_transform)) = cam_query.get_single() {
        if let Ok(mut scene_trans) = scene_transform.get_single_mut() {
            println!("Camera setup");

            let inverse = Transform::from_matrix((*cam_transform).compute_matrix().inverse());
            *scene_trans = *scene_trans * inverse;

            commands.spawn((
                Camera2dBundle {
                    camera: camera.clone(),
                    ..Default::default()
                },
                bevy_pancam::PanCam {
                    move_keys: DirectionKeys::NONE,
                    ..Default::default()
                },
            ));

            commands.entity(entity).remove::<Camera>();
        }
    };
}

#[allow(unused)]
fn play_animation(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat()
            .pause();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

fn keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    mut fs: ResMut<FrameStepper>,
) {
    for mut player in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };
        let playing_animation = player.animation_mut(playing_animation_index).unwrap();

        if keyboard_input.just_pressed(KeyCode::Space) {
            if playing_animation.is_paused() {
                playing_animation.resume();
                fs.is_animation_playing = true;
            } else {
                playing_animation.pause();
                fs.is_animation_playing = false;
            }
        }

        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            // Backward 1 frame
            fs.back();
            fs.shapes_buffer = None;

            println!(
                "LEFT seek: {} frame: {}",
                playing_animation.seek_time(),
                fs.current_frame
            );
        }

        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            // Forward 1 frame
            fs.forward();
            fs.shapes_buffer = None;

            println!(
                "RIGHT seek: {} frame: {}",
                playing_animation.seek_time(),
                fs.current_frame
            );
        }

        if playing_animation.is_paused() {
            playing_animation.seek_to(fs.current_frame as f32 / FRAME_RATE as f32);
        }
    }
}

fn export_lottie(world: &mut World) {
    println!("Exporting lottie...");
    let mut file = Lottie::new(FRAME_RATE);

    for frame in 0..FRAMES {
        println!("Frame: {}/{}", frame, FRAMES);

        world.run_system_once_with(frame, update_frame);
        world.run_system_once(update_animation);
        let shapes = world.run_system_once(get_shapes);

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

#[derive(Resource, Default)]
struct CachedMeshData {
    ids: Vec<AssetId<Mesh>>,
    meshes: Vec<Mesh>,
    colors: Vec<Color>,
    ordering: Vec<usize>,
}

// TODO: This only needs to run every time the frame changes
// PS: Caching only makes sense if the scene rotation is static
#[allow(clippy::complexity)]
fn cache_mesh_data(
    query: Query<(&GlobalTransform, &Handle<Mesh>, &Handle<StandardMaterial>), With<Handle<Mesh>>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut mesh_data: ResMut<CachedMeshData>,
) {
    // println!("Caching mesh data...");

    let mut ids = Vec::new();
    let mut t_meshes = Vec::new();
    let mut t_colors = Vec::new();

    for (global_transform, mesh_handle, material_handle) in query.iter() {
        let material = materials
            .get(material_handle)
            .expect("Mesh has no material");

        if let Some(mesh) = meshes.get(mesh_handle) {
            ids.push(mesh_handle.id());

            let transformed_mesh = mesh.clone().transformed_by((*global_transform).into());

            t_meshes.push(transformed_mesh);
            t_colors.push(material.base_color);
        }
    }

    if t_meshes.is_empty() {
        // println!("No meshes");
        return;
    }

    // println!("DONE");
    mesh_data.ids = ids;
    mesh_data.meshes = t_meshes;
    mesh_data.colors = t_colors;

    // *mesh_data = CachedMeshData {
    //     ids,
    //     meshes: t_meshes,
    //     colors: t_colors,
    //     ordering: Vec::new(),
    // };

    // Some((ids, t_meshes, t_colors))
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn update(
    mut scene: Query<&mut VelloScene>,
    mut fs: ResMut<FrameStepper>,
    mut highlight: ResMut<PathHighlight>,
    projection: Query<&OrthographicProjection>,
    mesh_data: Res<CachedMeshData>,
) {
    let mut scene = scene.single_mut();
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
