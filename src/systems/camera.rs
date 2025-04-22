use bevy::{prelude::*, scene::SceneInstance};
use bevy_pancam::DirectionKeys;
use bevy_vello::render::VelloView;

#[allow(clippy::type_complexity)]
pub fn camera_setup(
    mut commands: Commands,
    cam_query: Query<(Entity, &Camera, &GlobalTransform), (With<Camera3d>, Added<Camera3d>)>,
    mut scene_transform: Query<&mut Transform, (With<SceneInstance>, Without<Camera3d>)>,
) {
    if let Ok((_entity, camera, cam_transform)) = cam_query.get_single() {
        if let Ok(mut scene_trans) = scene_transform.get_single_mut() {
            println!("Camera setup");

            let inverse = Transform::from_matrix((*cam_transform).compute_matrix().inverse());
            // This is pointless
            *scene_trans = *scene_trans * inverse;

            let mut cam = camera.clone();
            cam.order = 1;

            commands.spawn((
                Camera2d,
                cam,
                // camera.clone(),
                bevy_pancam::PanCam {
                    move_keys: DirectionKeys::NONE,
                    ..Default::default()
                },
                VelloView,
            ));
        }
    };
}
