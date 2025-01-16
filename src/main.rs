use std::time::Duration;

use bevy::animation::animate_targets;
use bevy::core_pipeline::tonemapping::DebandDither;
use bevy::prelude::*;
use bevy_vello::{prelude::*, VelloPlugin};

use vello::AaConfig;

mod draw;
use draw::draw_collection;

mod edge;
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

// const GLB: &str = "exp2_mat.glb";
const GLB: &str = "p2.glb";

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
            [
                // GltfAssetLabel::Animation(2).from_asset(GLB),
                // GltfAssetLabel::Animation(1).from_asset(GLB),
                GltfAssetLabel::Animation(0).from_asset(GLB),
            ]
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
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
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
        // transitions
        //     .play(&mut player, animations.animations[0], Duration::ZERO)
        //     .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

#[allow(clippy::type_complexity)]
fn update(
    mut query_scene: Query<&mut VelloScene>,
    time: Res<Time>,
    meshes: ResMut<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut query: Query<
        (
            &GlobalTransform,
            &Handle<Mesh>,
            &Name,
            &Handle<StandardMaterial>,
        ),
        With<Handle<Mesh>>,
    >,
    mut trans: Query<&mut Transform, With<Penguin>>,
) {
    let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let mut scene = query_scene.single_mut();
    // Reset scene every frame
    *scene = VelloScene::default();

    let mut t_meshes = Vec::<Mesh>::new();
    let mut t_colors = Vec::<Color>::new();

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
        // t.rotate_y(0.003);
        // t.rotate_x(0.005);
    }

    for (t, m, name, mat_handle) in query.iter_mut() {
        if let Some(mesh) = meshes.get(m) {
            let material = materials.get(mat_handle).expect("Mesh has no material");
            // println!("{}", name);

            if name.as_str() != "Sphere.007" {
                continue;
            }

            let transformed_mesh = mesh.clone().transformed_by((*t).into());

            t_meshes.push(transformed_mesh);
            t_colors.push(material.base_color);

            // break;
        }
    }

    // println!("Mesh count: {}", count);

    draw_collection(t_meshes, t_colors, &mut scene);
}
