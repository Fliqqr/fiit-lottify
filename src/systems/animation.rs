use std::time::Duration;

use bevy::{animation::AnimationTarget, prelude::*};

use crate::{FrameStepper, FRAME_RATE};

#[derive(Resource, Default)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    pub graph: Handle<AnimationGraph>,
}

// pub fn get_animations(targets: Query<&AnimationTarget>, clips: Res<Assets<AnimationClip>>) {
//     println!("Clips: {}", clips.len());
//     println!("Getting all animation targets...");

//     for target in targets.iter() {
//         println!("Target: {:?}", target.id);
//     }

//     for clip in clips.iter() {
//         println!("Clip: {:?}", clip.1.duration());
//     }
// }

#[allow(unused)]
pub fn play_animation(
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

pub fn keyboard_control(
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
                println!("Resumed");
                fs.is_animation_playing = true;
            } else {
                playing_animation.pause();
                println!("Paused");
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
