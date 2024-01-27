use std::collections::HashMap;
use std::io::{Cursor, ErrorKind};
use std::path::Path;
use std::sync::Arc;

use bevy::math::{ivec3, vec2, Vec2};
use bevy::prelude::{Component, IVec3, Name, ResMut, Update, Vec3, Visibility};
use bevy::reflect::Reflect;
use bevy::render::color::Color;
use bevy::sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt},
    log,
    prelude::{
        Added, Asset, AssetApp, Assets, Bundle, Commands, GlobalTransform, Handle, Image, Plugin,
        Query, Res, Transform,
    },
    reflect::TypePath,
    utils::BoxedFuture,
};

use bevy_inspector_egui::prelude::*;
use bevy_simple_tilemap::{prelude::*, TileFlags};
use thiserror::Error;
use tiled::TileLayer;

use crate::movement::Moveable;

const SCALE: f32 = 3.0;

pub struct TilemapSize {
    pub columns: usize,
    pub rows: usize,
    pub width: usize,
    pub height: usize,
}

/// TimemapTileSize contains the width and height of a tile
#[derive(Component, Copy, Clone, Debug)]
pub struct TilemapTileSize {
    pub width: f32,
    pub height: f32,
}

impl TilemapTileSize {
    fn scaled(&self, scale: f32) -> Self {
        Self {
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

pub struct TilemapSpacing {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug)]
pub struct TileCollider;

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .register_asset_loader(TiledLoader)
            .register_type::<TiledMapBundle>()
            .add_systems(Update, process_loaded_maps);
    }
}

#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilemap_textures: HashMap<usize, Handle<Image>>,
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}

#[derive(Default, Bundle, Reflect)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

struct BytesResourceReader {
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Arc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(&mut self, _path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        // In this case, the path is ignored because the byte data is already provided.
        Ok(Cursor::new(self.bytes.clone()))
    }
}

pub struct TiledLoader;

#[derive(Debug, Error)]
pub enum TiledAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader::new(&bytes),
            );
            let map = loader.load_tmx_map(load_context.path()).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
            })?;

            let mut tilemap_textures = HashMap::default();
            let tile_image_offsets = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                if let Some(img) = &tileset.image {
                    // The load context path is the TMX file itself. If the file is at the root of the
                    // assets/ directory structure then the tmx_dir will be empty, which is fine.
                    let tmx_dir = load_context
                        .path()
                        .parent()
                        .expect("The asset load context was empty.");
                    let tile_path = tmx_dir.join(&img.source);
                    let asset_path = AssetPath::from(tile_path);
                    let texture: Handle<Image> = load_context.load(asset_path.clone());

                    tilemap_textures.insert(tileset_index, texture);
                }
            }

            let asset_map = TiledMap {
                map,
                tilemap_textures,
                tile_image_offsets,
            };

            log::info!("Loaded map: {}", load_context.path().display());

            Ok(asset_map)
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_query: Query<&Handle<TiledMap>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    maps: Res<Assets<TiledMap>>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
) {
    // If we have new map entities add them to the changed_maps list.
    for _new_map in new_maps.iter() {
        for map_handle in map_query.iter_mut() {
            if let Some(tiled_map) = maps.get(map_handle) {
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index)
                    else {
                        log::warn!("Skipped creating layer with missing tilemap textures.");
                        continue;
                    };

                    let tile_size = TilemapTileSize {
                        width: tileset.tile_width as f32,
                        height: tileset.tile_height as f32,
                    };

                    let tile_spacing = TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    };

                    let tilemap_size = TilemapSize {
                        columns: tileset.columns as usize,
                        rows: (tileset.tilecount / tileset.columns) as usize,
                        width: tiled_map.map.width as usize,
                        height: tiled_map.map.height as usize,
                    };

                    // Once materials have been created/added we need to then create the layers.
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        match layer.layer_type() {
                            tiled::LayerType::Tiles(tile_layer) => {
                                let Some(tiles) = build_tiles(
                                    &tile_layer,
                                    &tilemap_size,
                                    tileset_index,
                                    layer_index,
                                ) else {
                                    log::info!(
                                        "No tiles for layer {} [{}]",
                                        layer.name.clone(),
                                        layer_index,
                                    );
                                    continue;
                                };

                                let mut tilemap = TileMap::default();
                                tilemap.set_tiles(tiles);

                                // Spawn obstacles, this could do with putting somewhere nice, or
                                // merging into the build_tiles...
                                if let Some(obstacles) = build_obstacles(
                                    &tilemap_size,
                                    &tile_size,
                                    &tile_layer,
                                    layer_index,
                                ) {
                                    for (obstacle, obstacle_type) in obstacles {
                                        commands
                                            .spawn(SpriteBundle {
                                                sprite: Sprite {
                                                    color: Color::rgba(0.25, 0.25, 0.75, 0.5),
                                                    custom_size: Some(Vec2::new(
                                                        obstacle.width,
                                                        obstacle.height,
                                                    )),
                                                    ..Default::default()
                                                },
                                                transform: Transform {
                                                    translation: Vec3 {
                                                        x: obstacle.x,
                                                        y: obstacle.y,
                                                        z: 30.0,
                                                    },
                                                    ..Default::default()
                                                },
                                                // Set to visible if you want to see the collision
                                                // areas for debugging
                                                visibility: Visibility::Hidden,
                                                ..Default::default()
                                            })
                                            .insert(obstacle)
                                            .insert(obstacle_type);
                                    }
                                }

                                let texture_atlas = TextureAtlas::from_grid(
                                    tilemap_texture.clone(),
                                    vec2(tile_size.width, tile_size.height),
                                    tilemap_size.columns,
                                    tilemap_size.rows,
                                    Some(vec2(tile_spacing.x, tile_spacing.y)),
                                    None,
                                );

                                let texture_atlas_handle = texture_atlases.add(texture_atlas);
                                let translation = Vec3::new(
                                    -((tilemap_size.width as f32 * tile_size.scaled(SCALE).width)
                                        / 2.0)
                                        + ((tile_size.scaled(SCALE).width) / 2.0),
                                    -((tilemap_size.height as f32
                                        * tile_size.scaled(SCALE).height)
                                        / 2.0)
                                        + ((tile_size.scaled(SCALE).height) / 2.0),
                                    0.0,
                                );

                                let tilemap_bundle = TileMapBundle {
                                    tilemap,
                                    texture_atlas: texture_atlas_handle,
                                    transform: Transform {
                                        scale: Vec3::splat(SCALE),
                                        translation,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                };

                                let layer_name = layer.name.clone();

                                match layer_name.as_str() {
                                    "buildings" => {
                                        commands
                                            .spawn(tilemap_bundle)
                                            .insert(Name::new(layer_name))
                                            .insert(tile_size.scaled(SCALE))
                                            .insert(Buildings);
                                    }
                                    _ => {
                                        commands
                                            .spawn(tilemap_bundle)
                                            .insert(Name::new(layer_name));
                                    }
                                };
                            }
                            tiled::LayerType::Objects(object_layer) => {
                                let texture_atlas = TextureAtlas::from_grid(
                                    tilemap_texture.clone(),
                                    vec2(tile_size.width, tile_size.height),
                                    tilemap_size.columns,
                                    tilemap_size.rows,
                                    Some(vec2(tile_spacing.x, tile_spacing.y)),
                                    None,
                                );

                                let texture_atlas_handle = texture_atlases.add(texture_atlas);

                                for object in object_layer.objects() {
                                    // A sptite based tile that needs rendering
                                    if let Some(layer_tile_data) = object.tile_data() {
                                        let sprite_index = layer_tile_data.id();
                                        let sprite_x = (object.x * SCALE)
                                            - ((tilemap_size.width as f32
                                                * tile_size.scaled(SCALE).width)
                                                / 2.0);
                                        let sprite_y = -((object.y * SCALE)
                                            - ((tilemap_size.height as f32
                                                * tile_size.scaled(SCALE).height)
                                                / 2.0));
                                        let translation =
                                            Vec3::new(sprite_x, sprite_y, layer_index as f32);

                                        let sprite = TextureAtlasSprite::new(sprite_index as usize);

                                        let sprite_bundle = SpriteSheetBundle {
                                            texture_atlas: texture_atlas_handle.clone(),
                                            transform: Transform {
                                                scale: Vec3::splat(SCALE),
                                                translation,
                                                ..Default::default()
                                            },
                                            sprite,
                                            ..Default::default()
                                        };

                                        let layer_name = layer.name.clone();
                                        match layer_name.as_str() {
                                            "player" => {
                                                commands
                                                    .spawn(sprite_bundle)
                                                    .insert(Name::new(layer_name))
                                                    .insert(Moveable::new())
                                                    .insert(Player)
                                                    .insert(Inventory::new())
                                                    .insert(Size {
                                                        width: tile_size.scaled(SCALE).width,
                                                        height: tile_size.scaled(SCALE).height,
                                                    });
                                            }
                                            "princess" => {
                                                commands
                                                    .spawn(sprite_bundle)
                                                    .insert(Name::new(layer_name))
                                                    .insert(Moveable::new())
                                                    .insert(Princess);
                                            }
                                            "enemy" => {
                                                commands
                                                    .spawn(sprite_bundle)
                                                    .insert(Name::new(layer_name))
                                                    .insert(Moveable::new())
                                                    .insert(Enemy);
                                            }
                                            _ => {
                                                log::info!("Unknown layer name {}", layer_name);
                                                commands
                                                    .spawn(sprite_bundle)
                                                    .insert(Name::new(layer_name));
                                            }
                                        };
                                    } else {
                                        // A none sprite object with a shape
                                        // data: ObjectData { id: 31, tile: None, name: "Portal
                                        // Tunnel", user_type: "PortalTunnel", x: 464.0, y: 144.0,
                                        // rotation: 0.0, visible: true, shape: Rect { width: 16.0,
                                        // height: 64.0 }, properties: {} } }
                                        // We only care about recangle objects
                                        if object.user_type == "PortalTunnel" {
                                            let tiled::ObjectShape::Rect { width, height } =
                                                object.shape
                                            else {
                                                log::info!("Found non rectangle, skipping");
                                                continue;
                                            };

                                            let object_x = (object.x * SCALE)
                                                - ((tilemap_size.width as f32
                                                    * tile_size.scaled(SCALE).width)
                                                    / 2.0)
                                                + (width * 1.5); // this is because the x is in the
                                                                 // center of the rectangle, so we need to adjust for
                                                                 // that
                                            let object_y = -((object.y * SCALE)
                                                - ((tilemap_size.height as f32
                                                    * tile_size.scaled(SCALE).height)
                                                    / 2.0))
                                                - (height * 1.5); // this is because the y is in
                                                                  // the center of the rectangle, so we need to adjust
                                                                  // for that
                                            let translation =
                                                Vec3::new(object_x, object_y, layer_index as f32);

                                            commands
                                                .spawn(SpriteBundle {
                                                    sprite: Sprite {
                                                        color: Color::rgba(1., 1., 1., 0.5),
                                                        custom_size: Some(Vec2::new(width, height)),
                                                        ..Default::default()
                                                    },
                                                    transform: Transform {
                                                        scale: Vec3::splat(SCALE),
                                                        translation,
                                                        ..Default::default()
                                                    },
                                                    // Set to visible if you want to see the portal
                                                    // areas for debugging
                                                    visibility: Visibility::Hidden,
                                                    ..Default::default()
                                                })
                                                .insert(Portal::new())
                                                .insert(Size {
                                                    width: width * SCALE,
                                                    height: height * SCALE,
                                                })
                                                .insert(Name::new(object.user_type.clone()));
                                        }
                                    }
                                }
                            }
                            _ => (),
                        };
                    }
                }
            }
        }
    }
}

fn build_tiles(
    tile_layer: &TileLayer,
    tilemap_size: &TilemapSize,
    tileset_index: usize,
    layer_index: usize,
) -> Option<Vec<(IVec3, Option<Tile>)>> {
    log::info!("Building tile tiles for layer {}", layer_index);

    let tiled::TileLayer::Finite(layer_data) = tile_layer else {
        log::info!(
            "Skipping layer {} because only finite layers are supported.",
            layer_index,
        );
        return None;
    };

    let mut tiles: Vec<(IVec3, Option<Tile>)> = vec![];

    for x in 0..tilemap_size.width {
        for y in 0..tilemap_size.height {
            // Transform TMX coords into bevy coords.
            let mapped_y = tilemap_size.height - 1 - y;

            let mapped_x = x as i32;
            let mapped_y = mapped_y as i32;

            let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                Some(t) => t,
                None => {
                    continue;
                }
            };

            if tileset_index != layer_tile.tileset_index() {
                continue;
            }

            let layer_tile_data = match layer_data.get_tile_data(mapped_x, mapped_y) {
                Some(d) => d,
                None => {
                    continue;
                }
            };

            let flags = if layer_tile_data.flip_v && layer_tile_data.flip_d {
                TileFlags::FLIP_X | TileFlags::FLIP_Y
            } else if layer_tile_data.flip_v {
                TileFlags::FLIP_Y
            } else if layer_tile_data.flip_d {
                TileFlags::FLIP_X
            } else {
                TileFlags::default()
            };

            tiles.push((
                ivec3(x as i32, y as i32, layer_index as i32),
                Some(Tile {
                    sprite_index: layer_tile.id(),
                    flags,
                    ..Default::default()
                }),
            ));
        }
    }

    Some(tiles)
}

// The x, y should be a Point component
// the width, height should be a Size component
#[derive(Reflect, Component, Default, Debug, InspectorOptions)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Obstacle {
    pub fn new() -> Self {
        Self {
            x: 0.,
            y: 0.,
            width: 0.,
            height: 0.,
        }
    }
}

fn build_obstacles(
    tilemap_size: &TilemapSize,
    tile_size: &TilemapTileSize,
    tile_layer: &TileLayer,
    layer_index: usize,
) -> Option<Vec<(Obstacle, ObstacleType)>> {
    log::info!("Building obstacles for layer {}", layer_index);

    let tiled::TileLayer::Finite(layer_data) = tile_layer else {
        log::info!(
            "Skipping layer {} because only finite layers are supported.",
            layer_index,
        );
        return None;
    };

    let mut obstacles: Vec<(Obstacle, ObstacleType)> = vec![];

    for x in 0..tilemap_size.width {
        for y in 0..tilemap_size.height {
            // Transform TMX coords into bevy coords.
            let mapped_y = tilemap_size.height - 1 - y;

            let mapped_x = x as i32;
            let mapped_y = mapped_y as i32;

            let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                Some(t) => t,
                None => {
                    continue;
                }
            };

            // Extract obstacles. We are keeping this simple and only dealing
            // with Rect (rectangle) collision shapes.
            let Some(tile) = layer_tile.get_tile() else {
                continue;
            };

            let Some(collision) = &tile.collision else {
                continue;
            };

            log::info!("Tile data {:?}", &tile.user_type);

            let object_data = collision.object_data();

            for data in object_data.iter() {
                if let tiled::ObjectShape::Rect { width, height } = data.shape {
                    let obstacle_type = tile_user_class_to_component(&tile);

                    // TODO: Extract into some Point struct implementation?
                    // Convert x, y to screen coords
                    let width = width * SCALE;
                    let height = height * SCALE;
                    let x = (mapped_x as f32 * tile_size.scaled(SCALE).width)
                        - ((tilemap_size.width as f32 * tile_size.scaled(SCALE).width) / 2.0)
                        + (tile_size.scaled(SCALE).width / 2.0);
                    let y = -((mapped_y as f32 * tile_size.scaled(SCALE).height)
                        - ((tilemap_size.height as f32 * tile_size.scaled(SCALE).height) / 2.0))
                        - (tile_size.scaled(SCALE).height / 2.0);

                    let obstacle = Obstacle {
                        // TODO: add on the object.x to this value, as the collision shapes have an x within
                        x,
                        // TODO: add on the object.y to this value, as the collision shapes have a y within
                        y,
                        width,
                        height,
                    };

                    obstacles.push((obstacle, obstacle_type));
                }
            }
        }
    }

    if obstacles.is_empty() {
        log::info!("No obstacles found for layer {}", layer_index);
    }

    Some(obstacles)
}

fn tile_user_class_to_component(tile: &tiled::Tile) -> ObstacleType {
    match &tile.user_type {
        Some(obstacle_type) => match obstacle_type.as_str() {
            "Chest" => ObstacleType::Chest,
            "Potion" => ObstacleType::Potion,
            "Door" => ObstacleType::Door,
            "Tombstone" => ObstacleType::Tombstone,
            "Grave" => ObstacleType::Grave,
            "Fountain" => ObstacleType::Fountain,
            "Wall" => ObstacleType::Wall,
            _ => ObstacleType::None,
        },
        None => ObstacleType::None,
    }
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Princess;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Buildings;

#[derive(Component, Debug)]
pub struct Portal {
    pub entered: bool,
}

impl Portal {
    fn new() -> Self {
        Self { entered: false }
    }
}

#[derive(Component, Debug)]
pub struct Unknown;

#[derive(Component, Debug)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Reflect, Component, Default, Debug, InspectorOptions)]
pub enum ObstacleType {
    #[default]
    None,
    Chest,
    Potion,
    Wall,
    Door,
    Tombstone,
    Grave,
    Fountain,
}

#[derive(Component, Debug)]
pub struct Inventory {
    pub potion: ObstacleType,
    pub weapon: ObstacleType,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            potion: ObstacleType::None,
            weapon: ObstacleType::None,
        }
    }
}
