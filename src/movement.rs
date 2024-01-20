use bevy::{
    log,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use crate::tiled::{Obstacle, Player, Portal, Size};

const PLAYER_SPEED: f32 = 100.0;

#[derive(Debug)]
enum Direction {
    Stopped,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Debug)]
pub struct Moveable {
    speed: f32,
    direction: Direction,
}

impl Moveable {
    pub fn new() -> Self {
        Self {
            speed: 0.0,
            direction: Direction::Stopped,
        }
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                input_system_keyboard,
                input_system_touch,
                update_player_position,
                check_obstacle,
                check_portal,
            )
                .chain(),
        );
    }
}

fn input_system_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Moveable, With<Player>>,
) {
    let Ok(mut player_movement) = player_query.get_single_mut() else {
        return;
    };

    if keyboard_input.pressed(KeyCode::Up) {
        player_movement.direction = Direction::Up;
        player_movement.speed = PLAYER_SPEED;
        return;
    }

    if keyboard_input.pressed(KeyCode::Down) {
        player_movement.direction = Direction::Down;
        player_movement.speed = PLAYER_SPEED;
        return;
    }

    if keyboard_input.pressed(KeyCode::Left) {
        player_movement.direction = Direction::Left;
        player_movement.speed = PLAYER_SPEED;
        return;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        player_movement.direction = Direction::Right;
        player_movement.speed = PLAYER_SPEED;
        return;
    }

    // movement coasting
    if player_movement.speed > 0. {
        player_movement.speed -= 2.0;
    } else {
        player_movement.direction = Direction::Stopped;
        player_movement.speed = 0.0;
    }
}

fn update_player_position(
    mut player_query: Query<(&mut Transform, &Moveable), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut player_transform, player_movement)) = player_query.get_single_mut() else {
        return;
    };

    if matches!(player_movement.direction, Direction::Stopped) {
        return;
    }

    let movement_amount = player_movement.speed * time.delta_seconds();

    match player_movement.direction {
        Direction::Up => player_transform.translation.y += movement_amount,
        Direction::Down => player_transform.translation.y -= movement_amount,
        Direction::Left => player_transform.translation.x -= movement_amount,
        Direction::Right => player_transform.translation.x += movement_amount,
        _ => (),
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

fn check_obstacle(
    mut player_query: Query<(&mut Transform, &mut Moveable, &Size), With<Player>>,
    obstacle_query: Query<(&Transform, &Obstacle), Without<Player>>,
    portal_query: Query<&Portal>,
) {
    let Ok((mut player_transform, mut player_moveable, player_size)) =
        player_query.get_single_mut()
    else {
        return;
    };

    for (obstacle_transform, obstacle) in obstacle_query.iter() {
        if let Some(collision) = collide(
            player_transform.translation,
            Vec2::new(player_size.width, player_size.height),
            obstacle_transform.translation,
            Vec2::new(obstacle.width, obstacle.height),
        ) {
            // Moving left, collided with right side of wall
            if matches!(player_moveable.direction, Direction::Left)
                && matches!(collision, Collision::Right)
            {
                player_moveable.speed = 0.0;
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.x = obstacle_transform.translation.x + obstacle.width;
            };

            // Moving right, collided with left side of wall
            if matches!(player_moveable.direction, Direction::Right)
                && matches!(collision, Collision::Left)
            {
                player_moveable.speed = 0.0;
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.x = obstacle_transform.translation.x - obstacle.width;
            };

            // Moving up, collided with bottom side of wall
            if matches!(player_moveable.direction, Direction::Up)
                && matches!(collision, Collision::Bottom)
            {
                player_moveable.speed = 0.0;
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.y = obstacle_transform.translation.y - obstacle.height;
            };

            // Moving down, collided with top side of wall
            if matches!(player_moveable.direction, Direction::Down)
                && matches!(collision, Collision::Top)
            {
                for portal in portal_query.iter() {
                    if portal.entered {
                        log::info!("Enter portal");
                    }
                }
                player_moveable.speed = 0.0;
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.y = obstacle_transform.translation.y + obstacle.height;
            };
        }
    }
}

fn check_portal(
    mut player_query: Query<(&mut Transform, &Size), (With<Player>, Without<Portal>)>,
    mut portal_query: Query<(&Transform, &Size, &mut Portal), (With<Portal>, Without<Player>)>,
) {
    let Ok((mut player_transform, player_size)) = player_query.get_single_mut() else {
        return;
    };

    for (portal_transform, portal_size, mut portal) in portal_query.iter_mut() {
        portal.entered = collide(
            player_transform.translation,
            Vec2::new(player_size.width, player_size.height),
            portal_transform.translation,
            Vec2::new(portal_size.width, portal_size.height),
        )
        .is_some();
        if portal.entered {
            player_transform.translation.y =
                portal_transform.translation.y - (portal_size.height - (player_size.height * 1.5));
        }
    }
}
