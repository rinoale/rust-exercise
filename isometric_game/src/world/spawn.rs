use bevy::prelude::*;

use crate::gameplay::Station;
use crate::rendering::{create_diamond_mesh, create_rect_mesh, tile_sprite};
use crate::universe::{iso_translation, GridPosition, IsoRenderable, UniverseLayer};
use crate::world::blueprint::{PropVisual, TavernMap, WorldBlueprint};

pub fn spawn_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    blueprint: Res<WorldBlueprint>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for tile in &blueprint.tiles {
        let grid = tile.grid_vec();
        let layer = tile.kind.layer();
        commands.spawn((
            tile_sprite(asset_server.load(tile.kind.asset_path())),
            GridPosition::new(grid),
            IsoRenderable::new(0.0, layer),
            Transform::from_translation(iso_translation(grid, 0.0, layer, 0.0)),
        ));
    }

    for prop in &blueprint.props {
        spawn_prop(&mut commands, &mut meshes, &mut materials, prop.grid, &prop.visual);
    }

    for station in &blueprint.stations {
        spawn_prop(
            &mut commands,
            &mut meshes,
            &mut materials,
            station.prop.grid,
            &station.prop.visual,
        );
        commands.spawn((
            Station { kind: station.kind },
            GridPosition::new(station.prop.grid),
        ));
    }

    commands.spawn(TavernMap {
        seats: blueprint.seats.clone(),
        entrance: blueprint.entrance,
    });
}

fn spawn_prop(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: Vec2,
    visual: &PropVisual,
) {
    match visual {
        PropVisual::MeshRect {
            size,
            color,
            y_offset,
            layer,
            depth_offset,
            shadow,
        } => {
            if let Some(shadow) = shadow {
                commands.spawn((
                    Mesh2d(meshes.add(create_diamond_mesh(shadow.size.x, shadow.size.y))),
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(shadow.color))),
                    GridPosition::new(grid),
                    IsoRenderable::with_offset(
                        shadow.y_offset,
                        UniverseLayer::Decal,
                        shadow.depth_offset,
                    ),
                    Transform::from_translation(iso_translation(
                        grid,
                        shadow.y_offset,
                        UniverseLayer::Decal,
                        shadow.depth_offset,
                    )),
                ));
            }

            commands.spawn((
                Mesh2d(meshes.add(create_rect_mesh(size.x, size.y))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(*color))),
                GridPosition::new(grid),
                IsoRenderable::with_offset(*y_offset, *layer, *depth_offset),
                Transform::from_translation(iso_translation(
                    grid,
                    *y_offset,
                    *layer,
                    *depth_offset,
                )),
            ));
        }
    }
}
