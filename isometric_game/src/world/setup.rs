use bevy::prelude::*;

use crate::config::{
    MAP_HEIGHT, MAP_WIDTH, TAVERN_MAX_X, TAVERN_MAX_Y, TAVERN_MIN_X, TAVERN_MIN_Y, TILE_WIDTH,
};
use crate::gameplay::{Station, StationKind};
use crate::rendering::{create_diamond_mesh, create_rect_mesh, tile_sprite};
use crate::universe::{iso_translation, GridPosition, IsoRenderable, UniverseLayer};
use crate::world::layout::{is_tavern_floor, is_tavern_porch};
use crate::world::TavernMap;

pub fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let floor_tile = asset_server.load("tiles/tavern_floor.png");
    let porch_tile = asset_server.load("tiles/porch_wood.png");
    let ground_tile = asset_server.load("tiles/wasteland_ground.png");

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let grid = Vec2::new(x as f32, y as f32);
            let texture = tile_texture_for(x, y, &floor_tile, &porch_tile, &ground_tile);
            let layer = if is_tavern_floor(x, y) || is_tavern_porch(x, y) {
                UniverseLayer::Floor
            } else {
                UniverseLayer::Ground
            };
            commands.spawn((
                tile_sprite(texture),
                GridPosition::new(grid),
                IsoRenderable::new(0.0, layer),
                Transform::from_translation(iso_translation(grid, 0.0, layer, 0.0)),
            ));
        }
    }

    spawn_tavern_shell(&mut commands, &mut meshes, &mut materials);
    spawn_furniture(&mut commands, &mut meshes, &mut materials);

    commands.spawn(TavernMap {
        seats: vec![
            Vec2::new(11.2, 8.9),
            Vec2::new(13.0, 8.8),
            Vec2::new(15.1, 9.1),
            Vec2::new(12.0, 11.2),
            Vec2::new(14.1, 11.0),
            Vec2::new(16.4, 11.2),
        ],
        entrance: Vec2::new(20.8, 9.0),
    });
}

fn tile_texture_for(
    x: i32,
    y: i32,
    floor_tile: &Handle<Image>,
    porch_tile: &Handle<Image>,
    ground_tile: &Handle<Image>,
) -> Handle<Image> {
    if is_tavern_floor(x, y) {
        floor_tile.clone()
    } else if is_tavern_porch(x, y) {
        porch_tile.clone()
    } else {
        ground_tile.clone()
    }
}

fn spawn_tavern_shell(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let wall_mesh = meshes.add(create_rect_mesh(TILE_WIDTH * 0.9, 58.0));
    let wall_material = materials.add(ColorMaterial::from_color(Color::srgb(0.20, 0.15, 0.11)));
    let highlight_material =
        materials.add(ColorMaterial::from_color(Color::srgb(0.34, 0.26, 0.18)));

    for x in TAVERN_MIN_X..=TAVERN_MAX_X {
        let grid = Vec2::new(x as f32, TAVERN_MIN_Y as f32);
        commands.spawn((
            Mesh2d(wall_mesh.clone()),
            MeshMaterial2d(wall_material.clone()),
            GridPosition::new(grid),
            IsoRenderable::with_offset(34.0, UniverseLayer::Wall, 0.0),
            Transform::from_translation(iso_translation(grid, 34.0, UniverseLayer::Wall, 0.0)),
        ));
    }

    for y in TAVERN_MIN_Y + 1..=TAVERN_MAX_Y {
        let grid = Vec2::new(TAVERN_MIN_X as f32, y as f32);
        commands.spawn((
            Mesh2d(wall_mesh.clone()),
            MeshMaterial2d(highlight_material.clone()),
            GridPosition::new(grid),
            IsoRenderable::with_offset(30.0, UniverseLayer::Wall, -1.0),
            Transform::from_translation(iso_translation(grid, 30.0, UniverseLayer::Wall, -1.0)),
        ));
    }

    for x in [8.0, 12.0, 16.0] {
        spawn_prop(
            commands,
            meshes,
            materials,
            Vec2::new(x, 4.2),
            38.0,
            26.0,
            Color::srgb(0.08, 0.075, 0.06),
            35.0,
        );
    }

    spawn_prop(
        commands,
        meshes,
        materials,
        Vec2::new(20.2, 9.2),
        44.0,
        38.0,
        Color::srgb(0.26, 0.16, 0.10),
        33.0,
    );
}

fn spawn_furniture(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let wood = Color::srgb(0.34, 0.21, 0.11);
    let dark_wood = Color::srgb(0.20, 0.12, 0.07);
    let rust = Color::srgb(0.47, 0.20, 0.11);
    let canvas = Color::srgb(0.17, 0.22, 0.18);

    for x in 8..=15 {
        spawn_prop(
            commands,
            meshes,
            materials,
            Vec2::new(x as f32, 6.6),
            48.0,
            28.0,
            wood,
            42.0,
        );
    }

    spawn_station(
        commands,
        meshes,
        materials,
        StationKind::Stove,
        Vec2::new(8.1, 6.0),
        46.0,
        46.0,
        rust,
    );
    spawn_station(
        commands,
        meshes,
        materials,
        StationKind::Pantry,
        Vec2::new(6.4, 12.2),
        44.0,
        42.0,
        dark_wood,
    );
    spawn_station(
        commands,
        meshes,
        materials,
        StationKind::WaterStill,
        Vec2::new(9.2, 12.5),
        42.0,
        46.0,
        Color::srgb(0.22, 0.31, 0.34),
    );
    spawn_station(
        commands,
        meshes,
        materials,
        StationKind::Radio,
        Vec2::new(18.2, 6.3),
        34.0,
        36.0,
        Color::srgb(0.12, 0.16, 0.13),
    );

    for table in [
        Vec2::new(11.0, 9.4),
        Vec2::new(13.2, 9.4),
        Vec2::new(15.4, 9.4),
        Vec2::new(12.2, 11.6),
        Vec2::new(14.5, 11.6),
        Vec2::new(16.7, 11.6),
    ] {
        spawn_prop(commands, meshes, materials, table, 42.0, 20.0, wood, 36.0);
        spawn_prop(
            commands,
            meshes,
            materials,
            table + Vec2::new(-0.35, 0.35),
            22.0,
            16.0,
            dark_wood,
            35.0,
        );
    }

    for barricade in [
        Vec2::new(21.0, 5.0),
        Vec2::new(22.0, 6.0),
        Vec2::new(3.0, 15.0),
        Vec2::new(4.0, 16.0),
    ] {
        spawn_prop(commands, meshes, materials, barricade, 54.0, 22.0, canvas, 25.0);
    }
}

fn spawn_station(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    kind: StationKind,
    grid: Vec2,
    width: f32,
    height: f32,
    color: Color,
) {
    spawn_prop(commands, meshes, materials, grid, width, height, color, 45.0);
    commands.spawn((Station { kind }, GridPosition::new(grid)));
}

pub fn spawn_prop(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: Vec2,
    width: f32,
    height: f32,
    color: Color,
    layer: f32,
) {
    let shadow_mesh = meshes.add(create_diamond_mesh(width * 0.95, 13.0));
    let body_mesh = meshes.add(create_rect_mesh(width, height));
    commands.spawn((
        Mesh2d(shadow_mesh),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(0.0, 0.0, 0.0, 0.34)))),
        GridPosition::new(grid),
        IsoRenderable::with_offset(3.0, UniverseLayer::Decal, layer - 1.0),
        Transform::from_translation(iso_translation(grid, 3.0, UniverseLayer::Decal, layer - 1.0)),
    ));
    commands.spawn((
        Mesh2d(body_mesh),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
        GridPosition::new(grid),
        IsoRenderable::with_offset(height * 0.45, UniverseLayer::Prop, layer),
        Transform::from_translation(iso_translation(
            grid,
            height * 0.45,
            UniverseLayer::Prop,
            layer,
        )),
    ));
}
