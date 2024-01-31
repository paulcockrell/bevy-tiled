use std::fmt;

use bevy::{log, prelude::*, window::WindowResolution};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, InspectorOptions};
use bevy_simple_tilemap::prelude::*;
use hud::HudPlugin;
use movement::MovementPlugin;
use tiled_map::{
    TiledMap, TiledMapBundle, TiledMapPlugin, TiledObject, TiledShape, TilemapTileSize,
};

use crate::movement::Moveable;

mod hud;
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
        .add_plugins(HudPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            PostUpdate,
            (setup_player, setup_portals, setup_collectables),
        )
        .add_plugins(WorldInspectorPlugin::new())
        // Debugging
        .register_type::<Player>()
        .register_type::<Inventory>()
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
        match &tiled_object.class {
            Some(class) => {
                if class == "Player" {
                    commands
                        .entity(entity)
                        .insert(Player)
                        .insert(Inventory::default())
                        .insert(Moveable::new());
                }
            }
            _ => {
                commands.entity(entity);
            }
        }
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
        match &tiled_shape.class {
            Some(class) => {
                if class != "Portal" {
                    return;
                }

                let mut c = commands.entity(entity);
                c.insert(Portal).insert(Name::new(class.clone()));

                if let Some(name) = &tiled_shape.name {
                    let mut inventory = Inventory::new();

                    for n in name.split(',') {
                        match n {
                            "Green potion" => {
                                inventory.potion = Some(Collectable(Potion::Green));
                            }
                            "Red potion" => {
                                inventory.potion = Some(Collectable(Potion::Red));
                            }
                            "Blue potion" => {
                                inventory.potion = Some(Collectable(Potion::Blue));
                            }
                            "Hammer" => {
                                inventory.weapon = Some(Collectable(Weapon::Hammer));
                            }
                            "Axe" => {
                                inventory.weapon = Some(Collectable(Weapon::Axe));
                            }
                            "Sword" => {
                                inventory.weapon = Some(Collectable(Weapon::Sword));
                            }
                            _ => (),
                        }
                    }

                    c.insert(inventory);
                }
            }
            _ => {
                commands.entity(entity);
            }
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
        match &tiled_object.class {
            Some(class) => {
                if class == "Collectable" {
                    let mut c = commands.entity(entity);
                    c.insert(Name::new(class.clone()));

                    if let Some(name) = &tiled_object.name {
                        match name.as_str() {
                            "Green potion" => {
                                c.insert(Collectable(Potion::Green));
                            }
                            "Red potion" => {
                                c.insert(Collectable(Potion::Red));
                            }
                            "Blue potion" => {
                                c.insert(Collectable(Potion::Blue));
                            }
                            "Hammer" => {
                                c.insert(Collectable(Weapon::Hammer));
                            }
                            "Axe" => {
                                c.insert(Collectable(Weapon::Axe));
                            }
                            "Sword" => {
                                c.insert(Collectable(Weapon::Sword));
                            }
                            _ => (),
                        }
                    }
                }
            }
            _ => {
                commands.entity(entity);
            }
        };
    }

    log::info!("Setup collectables complete.");
}

#[derive(Component, Debug, Reflect, InspectorOptions)]
pub struct Player;

#[derive(Component, Debug, Reflect, InspectorOptions)]
pub struct Portal;

#[derive(Component, Debug, Reflect, InspectorOptions, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Collectable<T>(T);

#[derive(Component, Debug, Reflect, InspectorOptions, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Potion {
    Red,
    Green,
    Blue,
}

impl fmt::Display for Potion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Red => write!(f, "Red"),
            Self::Green => write!(f, "Green"),
            Self::Blue => write!(f, "Blue"),
        }
    }
}

#[derive(Component, Debug, Reflect, InspectorOptions, Clone, Copy, PartialEq, Eq)]
pub enum Weapon {
    Sword,
    Hammer,
    Axe,
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sword => write!(f, "Sword"),
            Self::Hammer => write!(f, "Hammer"),
            Self::Axe => write!(f, "Axe"),
        }
    }
}

// TODO: Inventory is probably better suited as a resource, as there is only one?
#[derive(Component, Debug, Reflect, InspectorOptions, Clone, Copy, PartialEq, Eq)]
pub struct Inventory {
    pub potion: Option<Collectable<Potion>>,
    pub weapon: Option<Collectable<Weapon>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            potion: None,
            weapon: None,
        }
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let potion = if let Some(collectable) = self.potion {
            collectable.0.to_string()
        } else {
            "Empty".to_string()
        };

        let weapon = if let Some(collectable) = self.weapon {
            collectable.0.to_string()
        } else {
            "Empty".to_string()
        };

        write!(f, "Inventory (Potion: {}, Weapon: {})", potion, weapon)
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
