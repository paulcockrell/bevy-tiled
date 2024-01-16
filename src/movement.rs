use bevy::{log, prelude::*, sprite::collide_aabb::collide};
use bevy_simple_tilemap::TileMap;

use crate::tiled::{Buildings, Player};

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
                check_wall,
                update_player_position,
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

    player_movement.direction = Direction::Stopped;
    player_movement.speed = 0.0;
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

fn check_wall(
    mut player_query: Query<(&mut Transform, &mut Moveable), With<Player>>,
    tilemap_query: Query<&TileMap, With<Buildings>>,
) {
    let Ok((mut player_transform, mut player_moveable)) = player_query.get_single_mut() else {
        return;
    };

    return;

    for tilemap in tilemap_query.iter() {
        for (_, chunk) in tilemap.chunks.iter() {
            for (index, tile) in chunk.tiles.iter().enumerate() {
                if tile.is_none() {
                    continue;
                }

                log::info!(
                    "index {} tile {:?} pos {}",
                    index,
                    tile,
                    // TODO: This totally doesn't work, we need a way
                    // to track where the building peices are, maybe spawn them as other entities
                    // with positions ? seems like duplicating stuff tho, as we have access to
                    // chunks and their tiles, but they don't store positions so it will have to
                    // be some mad calculations to deterime where they are on screen
                    row_major_pos(index)
                );

                // if let Some(collision) = collide(
                //     player_transform.translation,
                //     // TODO: Don't hard code this value, grab it from custom tile component thing,
                //     // should be in the query
                //     Vec2::new(48.0, 48.0),
                //     row_major_pos(index),
                //     Vec2::new(48.0, 48.0),
                // ) {
                //     log::info!("Ouch!!! {:?}", collision);
                // }
            }
        }

        // if let Some(collision) = collide(
        //     player_transform.translation,
        //     Vec2::new(tile_size.width, tile_size.height),
        //     wall_transform.translation,
        //     Vec2::new(tile_size.width, tile_size.height),
        // ) {
        //     // Moving left, collided with right side of wall
        //     if matches!(player_moveable.direction, Direction::Left)
        //         && matches!(collision, Collision::Right)
        //     {
        //         player_moveable.speed = 0.0;
        //         player_moveable.direction = Direction::Stopped;
        //         // Ensure we don't move in to the wall, as the collision may occur
        //         // after we have moved 'into' it (as translation is a vec3 of f32s)
        //         player_transform.translation.x = wall_transform.translation.x + tile_size.width;
        //     };

        //     // Moving right, collided with left side of wall
        //     if matches!(player_moveable.direction, Direction::Right)
        //         && matches!(collision, Collision::Left)
        //     {
        //         player_moveable.speed = 0.0;
        //         player_moveable.direction = Direction::Stopped;
        //         // Ensure we don't move in to the wall, as the collision may occur
        //         // after we have moved 'into' it (as translation is a vec3 of f32s)
        //         player_transform.translation.x = wall_transform.translation.x - tile_size.width;
        //     };

        //     // Moving up, collided with bottom side of wall
        //     if matches!(player_moveable.direction, Direction::Up)
        //         && matches!(collision, Collision::Bottom)
        //     {
        //         player_moveable.speed = 0.0;
        //         player_moveable.direction = Direction::Stopped;
        //         // Ensure we don't move in to the wall, as the collision may occur
        //         // after we have moved 'into' it (as translation is a vec3 of f32s)
        //         player_transform.translation.y = wall_transform.translation.y - tile_size.height;
        //     };

        //     // Moving down, collided with top side of wall
        //     if matches!(player_moveable.direction, Direction::Down)
        //         && matches!(collision, Collision::Top)
        //     {
        //         player_moveable.speed = 0.0;
        //         player_moveable.direction = Direction::Stopped;
        //         // Ensure we don't move in to the wall, as the collision may occur
        //         // after we have moved 'into' it (as translation is a vec3 of f32s)
        //         player_transform.translation.y = wall_transform.translation.y + tile_size.height;
        //     };
        // }
    }
}

/// Calculate row major position from index
pub fn row_major_pos(index: usize) -> Vec3 {
    let y = index / 64;
    Vec3::new((index - (y * 64)) as f32, y as f32, 0.0)
}
