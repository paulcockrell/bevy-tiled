use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use crate::{
    tiled_map::{TiledCollideable, TilemapTileSize},
    Collectable, Inventory, Player, Portal, Potion, Weapon,
};

const PLAYER_SPEED: f32 = 125.0;

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
                check_collideable,
                check_collectable_potion,
                check_collectable_weapon,
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

#[allow(clippy::type_complexity)]
fn check_collideable(
    mut player_query: Query<
        (&mut Transform, &mut Moveable, &TilemapTileSize),
        (With<Player>, Without<TiledCollideable>),
    >,
    collideable_query: Query<
        (&Transform, &TilemapTileSize),
        (With<TiledCollideable>, Without<Player>),
    >,
) {
    let Ok((mut player_transform, mut player_moveable, player_size)) =
        player_query.get_single_mut()
    else {
        return;
    };

    for (collideable_transform, collideable_size) in collideable_query.iter() {
        // TODO: The collideable size should be the width and height of the collideable shape, not
        // just the tile with and height
        if let Some(collision) = collide(
            player_transform.translation,
            Vec2::new(player_size.width, player_size.height),
            collideable_transform.translation,
            Vec2::new(collideable_size.width, collideable_size.height),
        ) {
            // Moving left, collided with right side of wall
            if matches!(player_moveable.direction, Direction::Left)
                && matches!(collision, Collision::Right)
            {
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.x =
                    collideable_transform.translation.x + collideable_size.width;
                player_moveable.speed = 0.0;
            };

            // Moving right, collided with left side of wall
            if matches!(player_moveable.direction, Direction::Right)
                && matches!(collision, Collision::Left)
            {
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.x =
                    collideable_transform.translation.x - collideable_size.width;
                player_moveable.speed = 0.0;
            };

            // Moving up, collided with bottom side of wall
            if matches!(player_moveable.direction, Direction::Up)
                && matches!(collision, Collision::Bottom)
            {
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.y =
                    collideable_transform.translation.y - collideable_size.height;
                player_moveable.speed = 0.0;
            };

            // Moving down, collided with top side of wall
            if matches!(player_moveable.direction, Direction::Down)
                && matches!(collision, Collision::Top)
            {
                // Ensure we don't move in to the wall, as the collision may occur
                // after we have moved 'into' it (as translation is a vec3 of f32s)
                player_transform.translation.y =
                    collideable_transform.translation.y + collideable_size.height;
                player_moveable.speed = 0.0;
            };
        }
    }
}

#[allow(clippy::type_complexity)]
fn check_collectable_potion(
    mut player_query: Query<
        (&mut Transform, &TilemapTileSize, &mut Inventory),
        (With<Player>, Without<Collectable<Potion>>),
    >,
    collectable_query: Query<
        (&Transform, &TilemapTileSize, &Collectable<Potion>),
        (With<Collectable<Potion>>, Without<Player>),
    >,
) {
    let Ok((player_transform, player_size, mut player_inventory)) = player_query.get_single_mut()
    else {
        return;
    };

    for (collectable_transform, collectable_size, collectable) in collectable_query.iter() {
        if let Some(_collision) = collide(
            player_transform.translation,
            Vec2::new(player_size.width, player_size.height),
            collectable_transform.translation,
            Vec2::new(collectable_size.width, collectable_size.height),
        ) {
            if player_inventory.potion != Some(*collectable) {
                player_inventory.potion = Some(*collectable);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn check_collectable_weapon(
    mut player_query: Query<
        (&mut Transform, &TilemapTileSize, &mut Inventory),
        (With<Player>, Without<Collectable<Weapon>>),
    >,
    collectable_query: Query<
        (&Transform, &TilemapTileSize, &Collectable<Weapon>),
        (With<Collectable<Weapon>>, Without<Player>),
    >,
) {
    let Ok((player_transform, player_size, mut player_inventory)) = player_query.get_single_mut()
    else {
        return;
    };

    for (collectable_transform, collectable_size, collectable) in collectable_query.iter() {
        if let Some(_collision) = collide(
            player_transform.translation,
            Vec2::new(player_size.width, player_size.height),
            collectable_transform.translation,
            Vec2::new(collectable_size.width, collectable_size.height),
        ) {
            if player_inventory.weapon != Some(*collectable) {
                player_inventory.weapon = Some(*collectable);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn check_portal(
    mut player_query: Query<
        (&mut Transform, &TilemapTileSize, &mut Moveable, &Inventory),
        (With<Player>, Without<Portal>),
    >,
    mut portal_query: Query<
        (&Transform, &TilemapTileSize, &mut Portal, &Inventory),
        (With<Portal>, Without<Player>),
    >,
) {
    let Ok((mut player_transform, player_size, mut player_moveable, player_inventory)) =
        player_query.get_single_mut()
    else {
        println!("Did not find player");
        return;
    };

    for (portal_transform, portal_size, _portal, portal_inventory) in portal_query.iter_mut() {
        if let Some(collision) = collide(
            player_transform.translation,
            Vec2::new(player_size.width, player_size.height),
            portal_transform.translation,
            Vec2::new(portal_size.width, portal_size.height),
        ) {
            if let (Some(pl_weapon), Some(pl_potion), Some(po_weapon), Some(po_potion)) = (
                player_inventory.weapon,
                player_inventory.potion,
                portal_inventory.weapon,
                portal_inventory.potion,
            ) {
                if pl_weapon == po_weapon && pl_potion == po_potion {
                    println!("You may pass, young warrior");
                } else {
                    println!("HALT!, you must collect the correct items to pass");
                    return;
                }
            } else {
                println!("HALT!, you must collect the correct items to pass");
                return;
            }

            match collision {
                Collision::Top => {
                    if matches!(player_moveable.direction, Direction::Down) {
                        player_transform.translation.y = portal_transform.translation.y
                            - (portal_size.height - (player_size.height * 1.5));
                        // Make the player 'pop' out the other side
                        player_moveable.speed = PLAYER_SPEED;
                    }
                }
                Collision::Bottom => {
                    if matches!(player_moveable.direction, Direction::Up) {
                        player_transform.translation.y = portal_transform.translation.y
                            + (portal_size.height - (player_size.height * 1.5));
                        // Make the player 'pop' out the other side
                        player_moveable.speed = PLAYER_SPEED;
                    }
                }
                _ => (),
            }
        }
    }
}
