use bevy::{log, prelude::*, window::WindowResolution};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, InspectorOptions};
use bevy_simple_tilemap::prelude::*;
use movement::MovementPlugin;
use tiled_map::{TiledMap, TiledMapBundle, TiledMapPlugin, TiledObject, TilemapTileSize};

use crate::movement::Moveable;

mod movement;
mod tiled_map;

pub const VIEW_WIDTH: f32 = 1600.0;
pub const VIEW_HEIGHT: f32 = 800.0;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(VIEW_WIDTH, VIEW_HEIGHT)
                            .with_scale_factor_override(1.0),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(SimpleTileMapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(MovementPlugin)
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, setup_objects)
        .add_plugins(WorldInspectorPlugin::new())
        // Debugging
        .register_type::<Chest>()
        .register_type::<TilemapTileSize>()
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");

    // TODO: If the tiled_map is spawned here... will all the other objects and sprites be spawned
    // even if this command below isn't executed!?
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn setup_objects(
    mut commands: Commands,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
    tiled_object_query: Query<(Entity, &TiledObject)>,
) {
    // Check to see if the maps were updated, if so continue to build objects else return
    if new_maps.is_empty() {
        return;
    }

    // for collideable_entity in collideable_entity_query.iter() {
    //     commands.entity(collideable_entity).log_components();
    // }
    for (entity, tiled_object) in tiled_object_query.iter() {
        log::info!("XXX Tiled Object Name {}", tiled_object.name);
        match tiled_object.name.as_str() {
            "Player" => commands
                .entity(entity)
                .insert(Player)
                .insert(Moveable::new()),
            "Wall" => commands.entity(entity).insert(Obstacle),
            "Chest" => commands.entity(entity).insert(Chest).insert(Obstacle),
            _ => &mut commands.entity(entity),
        };
    }

    log::info!("Setup map objects");
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Obstacle;

#[derive(Component, Debug, Reflect, InspectorOptions)]
pub struct Chest;

#[derive(Component, Debug)]
pub struct Princess;

#[derive(Component, Debug)]
pub struct Enemy;
