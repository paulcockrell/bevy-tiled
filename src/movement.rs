use bevy::prelude::*;

use crate::tiled::Player;

const PLAYER_SPEED: f32 = 100.0;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (input_system_keyboard, input_system_touch).chain());
    }
}

fn input_system_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
) {
    if let Some(mut tf) = player_transform_query.iter_mut().next() {
        if keyboard_input.pressed(KeyCode::Left) {
            tf.translation.x -= PLAYER_SPEED * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::Right) {
            tf.translation.x += PLAYER_SPEED * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Down) {
            tf.translation.y -= PLAYER_SPEED * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::Up) {
            tf.translation.y += PLAYER_SPEED * time.delta_seconds();
        }
    }
}

fn input_system_touch(
    touches: Res<Touches>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
) {
    for finger in touches.iter() {
        if touches.just_pressed(finger.id()) {
            println!("A new touch with ID {} just began.", finger.id());
        }
        println!(
            "Finger {} is at position ({},{}), started from ({},{}).",
            finger.id(),
            finger.position().x,
            finger.position().y,
            finger.start_position().x,
            finger.start_position().y,
        );
        // TODO: Convert touch to bevy coords, then detect if left, right, up, or down from player,
        // and set player movement direction to move one grid square in that direction
        if let Some(mut tf) = player_transform_query.iter_mut().next() {
            tf.translation.y = finger.position().y;
            tf.translation.x = finger.position().x;
        }
    }
}
