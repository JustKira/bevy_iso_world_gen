mod utils;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_mod_picking::prelude::*;
use fastnoise_lite::*;
use std::ops::Range;
use utils::tilemap_picker_backend::TilemapPickerBackend;

const WIDTH: u32 = 32;
const HEIGHT: u32 = 32;

#[derive(Component)]
struct PreviewBuilding {
    offset: Vec2,
}
// Define tile types as an enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileType {
    Dirt,
    Grass,
    Stone,
    Sand,
    Water,
    // Add more tile types as needed
}

// Implement a method to get the texture index for each tile type
impl TileType {
    fn texture_index(&self) -> u32 {
        match self {
            TileType::Dirt => 0,
            TileType::Grass => 1,
            TileType::Stone => 2,
            TileType::Sand => 3,
            TileType::Water => 4,
            // Add more mappings as needed
        }
    }
}

// Define a structure for tile type ranges
struct TileTypeRange {
    tile_type: TileType,
    range: Range<f32>,
}

#[derive(Resource)]
struct TileTypeRanges {
    ranges: Vec<TileTypeRange>,
}

impl TileTypeRanges {
    fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    fn add(&mut self, tile_type: TileType, start: f32, end: f32) {
        self.ranges.push(TileTypeRange {
            tile_type,
            range: start..end,
        });
    }

    fn get_tile_type(&self, value: f32) -> TileType {
        for range in &self.ranges {
            if range.range.contains(&value) {
                return range.tile_type;
            }
        }
        // Default tile type if no range matches
        TileType::Dirt
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -1000.0,
            scale: 0.5,
            far: 1000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // Set up tile type ranges
    let mut tile_type_ranges = TileTypeRanges::new();
    tile_type_ranges.add(TileType::Water, 0.0, 0.2);
    tile_type_ranges.add(TileType::Sand, 0.2, 0.375);
    tile_type_ranges.add(TileType::Dirt, 0.375, 0.45);
    tile_type_ranges.add(TileType::Grass, 0.45, 0.81);
    tile_type_ranges.add(TileType::Stone, 0.81, 1.0);

    commands.insert_resource(tile_type_ranges);
}

fn generate_world_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tile_type_ranges: Res<TileTypeRanges>,
) {
    let mut noise = FastNoiseLite::with_seed(1325);

    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(5));
    noise.set_frequency(Some(0.035));
    noise.set_fractal_weighted_strength(Some(-0.5));
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));

    // let mut noise_data = [[0.; HEIGHT as usize]; WIDTH as usize];

    let texture_handle: Handle<Image> = asset_server.load("iso.png");
    let map_size = TilemapSize {
        x: WIDTH as u32,
        y: HEIGHT as u32,
    };
    let tile_size = TilemapTileSize { x: 16.0, y: 17.0 };
    let grid_size = TilemapGridSize { x: 16.0, y: 8.0 };

    let map_type: TilemapType = TilemapType::Isometric(IsoCoordSystem::Diamond);

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let noise_value = noise.get_noise_2d(x as f32, y as f32);
            // noise_data[x][y] = (negative_1_to_1 + 1.) / 2.;

            let tile_pos = TilePos { x, y };

            let normalized_noise = (noise_value + 1.0) / 2.0; // Normalize to 0-1 range

            let tile_type = tile_type_ranges.get_tile_type(normalized_noise);

            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(tile_type.texture_index()),
                        ..Default::default()
                    },
                    On::<Pointer<Over>>::run(
                        move |event: Listener<Pointer<Over>>,
                              mut preview_query: Query<(&mut Transform, &PreviewBuilding)>,
                              tilemap_query: Query<(
                            &TilemapGridSize,
                            &TilemapType,
                            &GlobalTransform,
                        )>| {
                            let (grid_size, map_type, tilemap_transform) = tilemap_query.single();
                            let world_pos = tile_pos.center_in_world(grid_size, map_type);

                            if let Ok((mut preview_transform, preview_building)) =
                                preview_query.get_single_mut()
                            {
                                // let offset_world = tilemap_transform
                                //     .transform_point(preview_building.offset.extend(0.0));
                                preview_transform.translation =
                                    tilemap_transform.transform_point(world_pos.extend(10.0));
                                // + offset_world;
                            }
                        },
                    ),
                ))
                .id();

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        render_settings: TilemapRenderSettings {
            render_chunk_size: UVec2::new(2, 1),
            y_sort: true,
            ..Default::default()
        },
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
        ..Default::default()
    });
}

fn spawn_preview(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("h1(2x2).png"),
            transform: Transform::from_xyz(0.0, 0.0, 3.0),
            ..Default::default()
        },
        PreviewBuilding {
            offset: Vec2::new(0.0, 0.0),
        },
    ));
}

// fn preview_snap_to_tilemap(cameras: Query<(Entity, &Camera, &OrthographicProjection)>
//     tilemap_q: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &GlobalTransform)>,
// ) {}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TilemapPlugin,
            DefaultPickingPlugins,
            TilemapPickerBackend,
        ))
        .add_systems(Startup, (setup, generate_world_map, spawn_preview).chain())
        .run();
}
