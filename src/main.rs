use crate::tiled::TiledMapPlugin;
use bevy::{prelude::*, window::WindowResolution};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_simple_tilemap::prelude::*;
use tiled::Player;

mod tiled;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1280.0, 720.0)
                            .with_scale_factor_override(1.0),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(SimpleTileMapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, input_system)
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<tiled::TiledMap> = asset_server.load("map.tmx");

    commands.spawn(tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
) {
    const MOVE_SPEED: f32 = 5.0;

    if let Some(mut tf) = player_transform_query.iter_mut().next() {
        if keyboard_input.pressed(KeyCode::Left) {
            println!("WOOO LEFT");
            tf.translation.x -= MOVE_SPEED * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::Right) {
            println!("WOOO RIGHT");
            tf.translation.x += MOVE_SPEED * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Down) {
            println!("WOOO DOWN");
            tf.translation.y -= MOVE_SPEED * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::Up) {
            println!("WOOO UP");
            tf.translation.y += MOVE_SPEED * time.delta_seconds();
        }
    }
}
