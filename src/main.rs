use bevy::animation::animate_targets;
use bevy::core_pipeline::tonemapping::DebandDither;
use bevy::prelude::*;
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

#[derive(Component)]
struct Penguin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_plugins(VelloPlugin {
            antialiasing: AaConfig::Msaa16,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, update)
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
const GLB: &str = "double_donut.glb";
const FRAME_RATE: u64 = 30;
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
        Penguin,
    ));

    commands.spawn((
        Camera2dBundle {
            deband_dither: DebandDither::Enabled,
            ..Default::default()
        },
        bevy_pancam::PanCam::default(),
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

// Once the scene is loaded, start the animation
#[allow(clippy::type_complexity)]
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    query: Query<(&GlobalTransform, &Handle<Mesh>, &Handle<StandardMaterial>), With<Handle<Mesh>>>,
    meshes: ResMut<Assets<Mesh>>,
    lottie_fh: Option<ResMut<LottieFileHandler>>,
    materials: Res<Assets<StandardMaterial>>,
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

    if let Some(mut handle) = lottie_fh {
        if handle.frame >= FRAMES {
            if handle.frame > FRAMES {
                return;
            }
            handle.lottie.save_as(&format!("{}.json", GLB));
            handle.frame += 1;
        }

        println!("Frame: {}/{}", handle.frame, FRAMES);

        let mut t_meshes = Vec::new();
        let mut t_colors = Vec::new();

        for (t, m, mat) in query.iter() {
            let material = materials.get(mat).expect("Mesh has no material");

            if let Some(mesh) = meshes.get(m) {
                let transformed_mesh = mesh.clone().transformed_by((*t).into());

                t_meshes.push(transformed_mesh);
                t_colors.push(material.base_color);
            }
        }

        if t_meshes.is_empty() {
            return;
        }

        let mesh_shapes = generate_collection(t_meshes, t_colors);

        let frame = handle.frame;

        handle.lottie.add_layer(mesh_shapes, frame, frame + 1);
        handle.frame += 1;

        // let shapes = VectorShapes {
        //     shapes: mesh_shapes,
        // };
        // commands.insert_resource(shapes);
    }
}

#[allow(clippy::type_complexity)]
fn update(
    mut query_scene: Query<&mut VelloScene>,
    time: Res<Time>,
    vector_shapes: Option<Res<VectorShapes>>,
    mut trans: Query<&mut Transform>,
) {
    let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let mut scene = query_scene.single_mut();
    // Reset scene every frame
    *scene = VelloScene::default();

    // Animate color green to blue
    let c = Vec3::lerp(
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        sin_time + 0.5,
    );

    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::rgb(c.x as f64, c.y as f64, c.z as f64),
        None,
        &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, 0.0),
    );

    for mut t in trans.iter_mut() {
        // t.rotate_y(0.01);
        // t.rotate_x(0.01);
        // *t = Transform::from_rotation(
        //     Quat::from_rotation_y(4.1),
        //     // Quat::from_xyzw(3.0, 7.1, 0.0, 1.0),
        // );
    }

    if let Some(shapes) = vector_shapes {
        for mesh in &shapes.shapes {
            // println!("Got shapes: {:?}", mesh);

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
