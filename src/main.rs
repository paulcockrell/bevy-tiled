use bevy::{log, prelude::*, window::WindowResolution};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, InspectorOptions};
use bevy_simple_tilemap::prelude::*;
use movement::MovementPlugin;
use tiled_map::{
    TiledMap, TiledMapBundle, TiledMapPlugin, TiledObject, TiledShape, TilemapTileSize,
};

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
        .add_systems(
            PostUpdate,
            (setup_player, setup_portals, setup_collectables),
        )
        .add_plugins(WorldInspectorPlugin::new())
        // Debugging
        .register_type::<Player>()
        .register_type::<Collectable>()
        .register_type::<CollectableItem>()
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

fn setup_player(
    mut commands: Commands,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
    tiled_object_query: Query<(Entity, &TiledObject)>,
) {
    // Check to see if the maps were updated, if so continue to build objects else return
    if new_maps.is_empty() {
        return;
    }

    for (entity, tiled_object) in tiled_object_query.iter() {
        match tiled_object.name.as_str() {
            "Player" => commands
                .entity(entity)
                .insert(Player)
                .insert(Inventory::default())
                .insert(Moveable::new()),
            _ => &mut commands.entity(entity),
        };
    }

    log::info!("Setup player complete.");
}

fn setup_portals(
    mut commands: Commands,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
    tiled_shape_query: Query<(Entity, &TiledShape), With<TiledShape>>,
) {
    // Check to see if the maps were updated, if so continue to build objects else return
    if new_maps.is_empty() {
        return;
    }

    for (entity, tiled_shape) in tiled_shape_query.iter() {
        let Some(name) = &tiled_shape.name else {
            continue;
        };

        match name.as_str() {
            "PortalTunnel" => commands
                .entity(entity)
                .insert(Portal)
                .insert(Name::new(name.clone())),
            _ => commands.entity(entity).insert(Name::new(name.clone())),
        };
    }

    log::info!("Setup portal complete.");
}

fn setup_collectables(
    mut commands: Commands,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
    tiled_object_query: Query<(Entity, &TiledObject)>,
) {
    // Check to see if the maps were updated, if so continue to build objects else return
    if new_maps.is_empty() {
        return;
    }

    for (entity, tiled_object) in tiled_object_query.iter() {
        match tiled_object.name.as_str() {
            "RedChest" => commands
                .entity(entity)
                .insert(Collectable(CollectableItem::RedChest))
                .insert(Name::new(tiled_object.name.clone())),
            _ => &mut commands.entity(entity),
        };
    }

    log::info!("Setup collectables complete.");
}

#[derive(Component, Debug, Reflect, InspectorOptions, Clone, Copy, PartialEq, Eq)]
pub enum CollectableItem {
    RedChest,
}

#[derive(Component, Debug, Reflect, InspectorOptions, Clone, Copy, PartialEq, Eq)]
pub struct Collectable(CollectableItem);

#[derive(Component, Debug, Reflect, InspectorOptions)]
pub struct Player;

#[derive(Component, Debug, Reflect, InspectorOptions)]
pub struct Portal;

#[derive(Component, Debug, Reflect, InspectorOptions)]
pub struct Inventory(Vec<Collectable>);

impl Inventory {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_item(&mut self, new_item: Collectable) -> bool {
        if self.0.contains(&new_item) {
            return false;
        }
        self.0.push(new_item);
        true
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
