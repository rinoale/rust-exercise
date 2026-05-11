use bevy::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use rand::Rng;

const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;
const MAP_SIZE: i32 = 20;
const PLAYER_SPEED: f32 = 5.0;
const CAMERA_SPEED: f32 = 300.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Isometric Game".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<GameAssets>()
        .add_systems(Startup, (setup_camera, setup_map, setup_player))
        .add_systems(Update, (
            player_movement,
            camera_follow_player,
            camera_zoom,
        ))
        .run();
}

// --- Components ---

#[derive(Component)]
struct Player {
    grid_x: f32,
    grid_y: f32,
}

#[derive(Component)]
struct TilePos {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct MainCamera;

#[derive(Resource, Default)]
struct GameAssets;

// --- Coordinate conversion ---

fn grid_to_iso(gx: f32, gy: f32) -> Vec2 {
    Vec2::new(
        (gx - gy) * TILE_WIDTH * 0.5,
        (gx + gy) * TILE_HEIGHT * 0.5,
    )
}

fn tile_z_order(gx: f32, gy: f32) -> f32 {
    -(gx + gy)
}

// --- Tile colors ---

#[derive(Clone, Copy)]
enum TileType {
    Grass,
    Dirt,
    Water,
    Stone,
}

impl TileType {
    fn color(&self) -> Color {
        match self {
            TileType::Grass => Color::srgb(0.3, 0.7, 0.3),
            TileType::Dirt => Color::srgb(0.6, 0.45, 0.25),
            TileType::Water => Color::srgb(0.2, 0.4, 0.8),
            TileType::Stone => Color::srgb(0.5, 0.5, 0.5),
        }
    }
}

fn random_tile_type(rng: &mut impl Rng) -> TileType {
    match rng.gen_range(0..10) {
        0 => TileType::Water,
        1 => TileType::Dirt,
        2 => TileType::Stone,
        _ => TileType::Grass,
    }
}

// --- Systems ---

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
    ));
}

fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let diamond = create_iso_tile_mesh();
    let mesh_handle = meshes.add(diamond);
    let mut rng = rand::thread_rng();

    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let tile_type = random_tile_type(&mut rng);
            let iso_pos = grid_to_iso(x as f32, y as f32);
            let z = tile_z_order(x as f32, y as f32);

            let material = materials.add(ColorMaterial::from_color(tile_type.color()));

            commands.spawn((
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(material),
                Transform::from_translation(Vec3::new(iso_pos.x, -iso_pos.y, z)),
                TilePos { x, y },
            ));
        }
    }
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let start_x = MAP_SIZE as f32 / 2.0;
    let start_y = MAP_SIZE as f32 / 2.0;
    let iso_pos = grid_to_iso(start_x, start_y);

    let player_mesh = meshes.add(create_player_mesh());
    let player_material = materials.add(ColorMaterial::from_color(Color::srgb(0.9, 0.2, 0.2)));

    commands.spawn((
        Mesh2d(player_mesh),
        MeshMaterial2d(player_material),
        Transform::from_translation(Vec3::new(iso_pos.x, -iso_pos.y + 8.0, 100.0)),
        Player {
            grid_x: start_x,
            grid_y: start_y,
        },
    ));
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let (mut player, mut transform) = query.single_mut();

    let mut dx = 0.0f32;
    let mut dy = 0.0f32;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dx -= 1.0;
        dy -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dx += 1.0;
        dy += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dx -= 1.0;
        dy += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dx += 1.0;
        dy -= 1.0;
    }

    if dx != 0.0 || dy != 0.0 {
        let len = (dx * dx + dy * dy).sqrt();
        dx /= len;
        dy /= len;

        player.grid_x += dx * PLAYER_SPEED * time.delta_secs();
        player.grid_y += dy * PLAYER_SPEED * time.delta_secs();

        player.grid_x = player.grid_x.clamp(0.0, (MAP_SIZE - 1) as f32);
        player.grid_y = player.grid_y.clamp(0.0, (MAP_SIZE - 1) as f32);

        let iso_pos = grid_to_iso(player.grid_x, player.grid_y);
        transform.translation.x = iso_pos.x;
        transform.translation.y = -iso_pos.y + 8.0;
        transform.translation.z = 100.0;
    }
}

fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    let target = Vec3::new(
        player_transform.translation.x,
        player_transform.translation.y,
        camera_transform.translation.z,
    );

    camera_transform.translation = camera_transform
        .translation
        .lerp(target, CAMERA_SPEED * time.delta_secs() / 100.0);
}

fn camera_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut projection = camera_query.single_mut();

    for event in scroll_events.read() {
        let scroll = match event.unit {
            MouseScrollUnit::Line => event.y * 0.1,
            MouseScrollUnit::Pixel => event.y * 0.001,
        };
        projection.scale = (projection.scale - scroll).clamp(0.3, 3.0);
    }
}

// --- Mesh builders ---

fn create_iso_tile_mesh() -> Mesh {
    let hw = TILE_WIDTH * 0.5;
    let hh = TILE_HEIGHT * 0.5;

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, hh, 0.0],   // top
            [hw, 0.0, 0.0],   // right
            [0.0, -hh, 0.0],  // bottom
            [-hw, 0.0, 0.0],  // left
        ],
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![
        0, 1, 2,
        0, 2, 3,
    ]))
}

fn create_player_mesh() -> Mesh {
    let w = 12.0;
    let h = 20.0;

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-w, 0.0, 0.0],
            [w, 0.0, 0.0],
            [w, h, 0.0],
            [-w, h, 0.0],
        ],
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![
        0, 1, 2,
        0, 2, 3,
    ]))
}
