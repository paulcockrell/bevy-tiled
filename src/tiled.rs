use std::collections::HashMap;
use std::io::{Cursor, ErrorKind};
use std::path::Path;
use std::sync::Arc;

use bevy::math::{ivec3, vec2};
use bevy::prelude::{Component, IVec3, Name, ResMut, Update, Vec3};
use bevy::sprite::TextureAtlas;
use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt},
    log,
    prelude::{
        Added, Asset, AssetApp, AssetEvent, AssetId, Assets, Bundle, Commands, EventReader,
        GlobalTransform, Handle, Image, Plugin, Query, Res, Transform,
    },
    reflect::TypePath,
    utils::BoxedFuture,
};

use bevy_simple_tilemap::{prelude::*, TileFlags};
use thiserror::Error;
use tiled::{ObjectLayer, TileLayer};

pub struct TilemapSize {
    pub columns: usize,
    pub rows: usize,
    pub width: usize,
    pub height: usize,
}

pub struct TilemapTileSize {
    pub x: f32,
    pub y: f32,
}

pub struct TilemapSpacing {
    pub x: f32,
    pub y: f32,
}

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .register_asset_loader(TiledLoader)
            .add_systems(Update, process_loaded_maps);
    }
}

#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilemap_textures: HashMap<usize, Handle<Image>>,
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}

#[derive(Default, Bundle)]
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
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    mut map_query: Query<&Handle<TiledMap>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    maps: Res<Assets<TiledMap>>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
) {
    let mut changed_maps = Vec::<AssetId<TiledMap>>::default();

    // TODO: Add in system to remove existing map

    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                log::info!("Map added!");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                log::info!("Map changed!");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.id());
    }

    for changed_map in changed_maps.iter() {
        for map_handle in map_query.iter_mut() {
            // only deal with currently changed map
            if map_handle.id() != *changed_map {
                continue;
            }

            if let Some(tiled_map) = maps.get(map_handle) {
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index)
                    else {
                        log::warn!("Skipped creating layer with missing tilemap textures.");
                        continue;
                    };

                    let tile_size = TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
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
                        let tiles = match layer.layer_type() {
                            tiled::LayerType::Tiles(tile_layer) => {
                                build_tiles(&tilemap_size, tileset_index, layer_index, tile_layer)
                            }
                            tiled::LayerType::Objects(object_layer) => {
                                build_objects(layer_index, object_layer)
                            }
                            _ => None,
                        };

                        let Some(tiles) = tiles else {
                            println!("No tiles for layer {}", layer_index);
                            continue;
                        };

                        let mut tilemap = TileMap::default();
                        tilemap.set_tiles(tiles);

                        let texture_atlas = TextureAtlas::from_grid(
                            tilemap_texture.clone(),
                            vec2(tile_size.x, tile_size.y),
                            tilemap_size.columns,
                            tilemap_size.rows,
                            Some(vec2(tile_spacing.x, tile_spacing.y)),
                            None,
                        );

                        let texture_atlas_handle = texture_atlases.add(texture_atlas);

                        let tilemap_bundle = TileMapBundle {
                            tilemap,
                            texture_atlas: texture_atlas_handle,
                            transform: Transform {
                                scale: Vec3::splat(3.0),
                                translation: Vec3::new(0.0, 0.0, 0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        let layer_name = layer.name.clone();

                        match layer_name.as_str() {
                            "player" => {
                                commands
                                    .spawn(tilemap_bundle)
                                    .insert(Name::new(layer_name))
                                    .insert(Player);
                            }
                            "wall" => {
                                commands
                                    .spawn(tilemap_bundle)
                                    .insert(Name::new(layer_name))
                                    .insert(Wall);
                            }
                            _ => {
                                commands.spawn(tilemap_bundle).insert(Name::new(layer_name));
                            }
                        };
                    }
                }
            }
        }
    }
}

fn build_tiles(
    tilemap_size: &TilemapSize,
    tileset_index: usize,
    layer_index: usize,
    tile_layer: TileLayer,
) -> Option<Vec<(IVec3, Option<Tile>)>> {
    println!("Building tile tiles for layer {}", layer_index);

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

fn build_objects(
    layer_index: usize,
    object_layer: ObjectLayer,
) -> Option<Vec<(IVec3, Option<Tile>)>> {
    println!("Building object tiles for layer {}", layer_index);

    let mut tiles: Vec<(IVec3, Option<Tile>)> = vec![];

    for object in object_layer.objects() {
        // let Some(tile) = object.get_tile() else {
        //     println!("No tile found, skipping");
        //     continue;
        // };

        let Some(layer_tile_data) = object.tile_data() else {
            println!("No tile data found, skipping");
            continue;
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

        print!(
            "x {}\ny {}\nlyr_idx {}\nsprite_idx {}\nflags {:?}\n\n",
            object.x,
            object.y,
            layer_index,
            layer_tile_data.id(),
            flags
        );

        tiles.push((
            ivec3(1, 1, layer_index as i32),
            Some(Tile {
                sprite_index: layer_tile_data.id(),
                flags,
                ..Default::default()
            }),
        ));
    }

    Some(tiles)
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Wall;

#[derive(Component, Debug)]
pub struct Unknown;
