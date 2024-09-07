use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use fastnoise_lite::*;

const WIDTH: u32 = 32;
const HEIGHT: u32 = 32;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, (generate_world_map, setup).chain())
        .run();
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
}

fn generate_world_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut noise = FastNoiseLite::with_seed(1337);

    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(4));
    noise.set_frequency(Some(0.05));
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
            let negative_1_to_1 = noise.get_noise_2d(x as f32, y as f32);
            // noise_data[x][y] = (negative_1_to_1 + 1.) / 2.;

            let tile_pos = TilePos { x, y };

            let tile_index = if negative_1_to_1 < -0.5 {
                2
            } else if negative_1_to_1 < -0.25 {
                0
            } else {
                1
            };

            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(tile_index),
                    ..Default::default()
                })
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
            render_chunk_size: UVec2::new(3, 1),
            y_sort: true,
            ..Default::default()
        },
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
        ..Default::default()
    });
}
