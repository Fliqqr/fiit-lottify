use bevy::prelude::*;
use bevy::{animation::animate_targets, scene::SceneInstance};
use bevy_vello::vello::wgpu::core::device::global;
use bevy_vello::{prelude::*, VelloPlugin};
use std::time::Duration;

use kurbo::Affine;
use lottie::Lottie;
use vello::AaConfig;

mod draw;
use draw::{generate_collection, MeshShape};

mod lottie;

/*
https://github.com/zimond/lottie-rs/
https://lottie.github.io/lottie-spec/1.0/single-page/
*/

/*
TODO:

1. Rotate scene to align camera with the viewport

2. GUI

3. Preview and framestepping

4. Exporting frame as SVG
*/

#[derive(Component)]
struct GltfScene;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_plugins(VelloPlugin {
            antialiasing: AaConfig::Msaa16,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(SpawnScene, camera_setup)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        // .add_systems(Update, update)
        .add_systems(Update, camera_setup)
        .run();
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

#[derive(Resource)]
struct VectorShapes {
    shapes: Vec<MeshShape>,
}

#[derive(Resource)]
struct LottieFileHandler {
    lottie: Lottie,
    frame: u64,
}

// const GLB: &str = "ico.glb";
const GLB: &str = "camera2.glb";
const FRAME_RATE: u64 = 1;
const FRAMES: u64 = 1;

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

    commands.insert_resource(LottieFileHandler {
        lottie: Lottie::new(FRAME_RATE),
        frame: 0,
    });
}

#[allow(clippy::type_complexity)]
fn camera_setup(
    mut commands: Commands,
    cam_query: Query<(Entity, &Camera, &GlobalTransform), With<Camera3d>>,
    mut scene_transform: Query<&mut Transform, (With<SceneInstance>, Without<Camera3d>)>,
) {
    // println!("Camera setup");

    if let Ok((entity, camera, cam_transform)) = cam_query.get_single() {
        // println!("Found {:?}", camera);
        // println!("{:?}", cam_transform);

        if let Ok(mut scene_trans) = scene_transform.get_single_mut() {
            // println!("Old scene {:?}", scene_trans);

            let inverse = Transform::from_matrix((*cam_transform).compute_matrix().inverse());
            *scene_trans = *scene_trans * inverse;

            // println!("Rotated scene: {:?}", scene_trans);
            // println!("Rotated camera: {:?}", cam_transform);

            commands.spawn((
                Camera2dBundle {
                    camera: camera.clone(),
                    // transform: (*transform).into(),
                    ..Default::default()
                },
                bevy_pancam::PanCam::default(),
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
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

fn save_to_lottie_file(mut handle: LottieFileHandler) {
    if handle.frame >= FRAMES {
        if handle.frame > FRAMES {
            return;
        }
        handle.lottie.save_as(&format!("{}.json", GLB));
        handle.frame += 1;
    }
}

#[allow(clippy::complexity)]
fn load_mesh_data(
    query: Query<(&GlobalTransform, &Handle<Mesh>, &Handle<StandardMaterial>), With<Handle<Mesh>>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) -> Option<(Vec<Mesh>, Vec<Color>)> {
    let mut t_meshes = Vec::new();
    let mut t_colors = Vec::new();

    for (global_transform, mesh_handle, material_handle) in query.iter() {
        let material = materials
            .get(material_handle)
            .expect("Mesh has no material");

        if let Some(mesh) = meshes.get(mesh_handle) {
            let transformed_mesh = mesh.clone().transformed_by((*global_transform).into());

            t_meshes.push(transformed_mesh);
            t_colors.push(material.base_color);
        }
    }

    if t_meshes.is_empty() {
        return None;
    }

    Some((t_meshes, t_colors))
}

// Once the scene is loaded, start the animation
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn setup_scene_once_loaded(
    // mut commands: Commands,
    mut scene: Query<&mut VelloScene>,

    query: Query<(&GlobalTransform, &Handle<Mesh>, &Handle<StandardMaterial>), With<Handle<Mesh>>>,
    // lottie_fh: Option<ResMut<LottieFileHandler>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    let mut scene = scene.single_mut();
    *scene = VelloScene::default();

    if let Some((t_meshes, t_colors)) = load_mesh_data(query, meshes, materials) {
        let mesh_shapes = generate_collection(t_meshes, t_colors);

        for mesh in mesh_shapes {
            let color = mesh.color.to_linear();

            scene.fill(
                peniko::Fill::NonZero,
                // &Stroke {
                //     width: 0.02,
                //     ..Default::default()
                // },
                Affine::IDENTITY,
                peniko::Color::rgb(color.red.into(), color.green.into(), color.blue.into()),
                None,
                &mesh.shape.paths.as_slice(),
            );
        }
    }
}

// #[allow(clippy::type_complexity)]
// fn update(
//     mut query_scene: Query<&mut VelloScene>,
//     // time: Res<Time>,
//     vector_shapes: Option<Res<VectorShapes>>,
// ) {
//     // let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
//     let mut scene = query_scene.single_mut();
//     // Reset scene every frame
//     *scene = VelloScene::default();

//     if let Some(shapes) = vector_shapes {
//         for mesh in &shapes.shapes {
//             // println!("Got shapes: {:?}", mesh);

//             let color = mesh.color.to_linear();

//             scene.fill(
//                 peniko::Fill::NonZero,
//                 // &Stroke {
//                 //     width: 0.02,
//                 //     ..Default::default()
//                 // },
//                 Affine::IDENTITY,
//                 peniko::Color::rgb(color.red.into(), color.green.into(), color.blue.into()),
//                 None,
//                 &mesh.shape.paths.as_slice(),
//             );
//         }
//     }
// }
