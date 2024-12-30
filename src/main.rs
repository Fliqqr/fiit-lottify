use bevy::core_pipeline::tonemapping::DebandDither;
use bevy::prelude::*;
use bevy_vello::{prelude::*, VelloPlugin};

use vello::AaConfig;

mod draw;
use draw::draw_collection;

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
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("penguin.gltf")),
            ..default()
        },
        Penguin,
    ));

    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });
    // // camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });

    commands.spawn((
        Camera2dBundle {
            deband_dither: DebandDither::Enabled,
            ..Default::default()
        },
        bevy_pancam::PanCam::default(),
    ));

    commands.spawn(VelloSceneBundle::default());
}

fn update(
    mut query_scene: Query<&mut VelloScene>,
    time: Res<Time>,
    meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&GlobalTransform, &Handle<Mesh>), With<Handle<Mesh>>>,
    mut trans: Query<&mut Transform, With<Penguin>>,
) {
    let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let mut scene = query_scene.single_mut();
    // Reset scene every frame
    *scene = VelloScene::default();

    let mut t_meshes = Vec::<Mesh>::new();

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
        t.rotate_y(0.1);
    }

    for (t, m) in query.iter_mut() {
        if let Some(mesh) = meshes.get(m) {
            let transformed_mesh = mesh.clone().transformed_by((*t).into());

            // let positions = transformed_mesh
            //     .attribute(Mesh::ATTRIBUTE_POSITION)
            //     .unwrap()
            //     .as_float3()
            //     .expect("`Mesh::ATTRIBUTE_POSITION` vertex attributes should be of type `float3`");

            // println!("{:?}", positions[0]);

            t_meshes.push(transformed_mesh);
        }
    }

    draw_collection(t_meshes, &mut scene);

    // println!("--");
}
