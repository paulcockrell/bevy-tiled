use bevy::{
    math::{ivec3, vec2},
    prelude::*,
    window::WindowResolution,
};
use bevy_simple_tilemap::prelude::*;
use bevy_simple_tilemap::TileFlags;
use tiled::Loader;

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
        .add_systems(Startup, setup)
        .add_systems(Update, input_system)
        .run();
}

fn input_system(
    mut camera_transform_query: Query<&mut Transform, With<Camera2d>>,
    mut tilemap_visible_query: Query<&mut Visibility, With<TileMap>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    const MOVE_SPEED: f32 = 100.0;
    const ZOOM_SPEED: f32 = 10.0;

    if let Some(mut tf) = camera_transform_query.iter_mut().next() {
        if keyboard_input.pressed(KeyCode::X) {
            tf.scale -= Vec3::splat(ZOOM_SPEED) * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::Z) {
            tf.scale += Vec3::splat(ZOOM_SPEED) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::A) {
            tf.translation.x -= MOVE_SPEED * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::D) {
            tf.translation.x += MOVE_SPEED * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::S) {
            tf.translation.y -= MOVE_SPEED * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::W) {
            tf.translation.y += MOVE_SPEED * time.delta_seconds();
        }

        if keyboard_input.just_pressed(KeyCode::V) {
            // Toggle visibility
            let mut visibility = tilemap_visible_query.iter_mut().next().unwrap();

            if *visibility == Visibility::Hidden {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Load tilesheet texture and make a texture atlas from it
    let texture_handle = asset_server.load("tilemap_packed.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        vec2(16.0, 16.0),
        12,
        11,
        Some(vec2(0.0, 0.0)),
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let mut tiles: Vec<(IVec3, Option<Tile>)> = vec![];
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/map.tmx").unwrap();

    // let tile_layers = map.layers().filter_map(|layer| match layer.layer_type() {
    //     tile::LayerType::Tiles(layer) => Some(layer),
    //     _ => None,
    // });

    // for layer in tile_layers {
    //     layer_renderer(layer);
    // }

    for (i, layer) in map.layers().enumerate() {
        print!(
            "Layer [{}] \"{}\"\n\t {} {}",
            i, layer.name, layer.offset_x, layer.offset_y
        );
        match layer.layer_type() {
            tiled::LayerType::Tiles(layer) => match layer {
                tiled::TileLayer::Finite(data) => {
                    println!("Layer width {} height {}", map.width, map.height);
                    for x in 0..map.width {
                        for y in 0..map.height {
                            // Transform TMX coords into bevy coords.
                            let mapped_x = x as i32;
                            let mapped_y = map.height - 1 - y;
                            let mapped_y = mapped_y as i32;

                            let layer_tile = match data.get_tile(mapped_x, mapped_y) {
                                Some(t) => t,
                                None => {
                                    continue;
                                }
                            };

                            let layer_tile_data = match data.get_tile_data(mapped_x, mapped_y) {
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
                                ivec3(x as i32, y as i32, i as i32), // i is used to set the Z of the
                                // tile, this effectively states which 'layer' the tile is on.
                                Some(Tile {
                                    sprite_index: layer_tile.id(),
                                    flags,
                                    ..Default::default()
                                }),
                            ));
                        }
                    }
                }
                _ => println!("Not finite layer, not supported"),
            },
            tiled::LayerType::Objects(layer) => {
                println!("Object layer with {} objects", layer.objects().len())
            }
            _ => println!("Other layer type, no supported"),
        }
    }

    let mut tilemap = TileMap::default();
    tilemap.set_tiles(tiles);

    // Set up tilemap
    let tilemap_bundle = TileMapBundle {
        tilemap,
        texture_atlas: texture_atlas_handle.clone(),
        transform: Transform {
            scale: Vec3::splat(3.0),
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    };

    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Spawn tilemap
    commands.spawn(tilemap_bundle);
}
