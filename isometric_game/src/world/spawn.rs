use bevy::prelude::*;

use crate::gameplay::{Station, StationCooldown};
use crate::rendering::{create_diamond_mesh, create_rect_mesh, tile_sprite};
use crate::universe::{GridPosition, IsoRenderable, UniverseLayer, iso_translation};
use crate::world::blueprint::{CutawayRule, PropPlacement, PropVisual, TavernMap, WorldBlueprint};
use crate::world::cutaway::CutawayWhenPlayerInside;

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
            tile_sprite(asset_server.load(tile.kind.asset_path(tile.grid))),
            GridPosition::new(grid),
            IsoRenderable::new(0.0, layer),
            Transform::from_translation(iso_translation(grid, 0.0, layer, 0.0)),
        ));
    }

    for prop in &blueprint.props {
        spawn_prop(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            prop,
        );
    }

    for station in &blueprint.stations {
        spawn_prop(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            &station.prop,
        );
        commands.spawn((
            Station { kind: station.kind },
            StationCooldown::default(),
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
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    prop: &PropPlacement,
) {
    let grid = prop.grid;

    match &prop.visual {
        PropVisual::MeshRect {
            size,
            color,
            anchor,
            layer,
            depth_offset,
            shadow,
        } => {
            let y_offset = anchor.y_offset(*size);

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

            let mut body = commands.spawn((
                Mesh2d(meshes.add(create_rect_mesh(size.x, size.y))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(*color))),
                GridPosition::new(grid),
                Visibility::default(),
                IsoRenderable::with_offset(y_offset, *layer, *depth_offset),
                Transform::from_translation(iso_translation(grid, y_offset, *layer, *depth_offset)),
            ));

            if let CutawayRule::HideWhenPlayerInside { zone } = &prop.cutaway {
                body.insert(CutawayWhenPlayerInside { zone: zone.clone() });
            }
        }
        PropVisual::Sprite {
            asset_path,
            size,
            anchor,
            layer,
            depth_offset,
            shadow,
        } => {
            let y_offset = anchor.y_offset(*size);

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

            let mut sprite = Sprite::from_image(asset_server.load(asset_path.clone()));
            sprite.custom_size = Some(*size);
            let mut body = commands.spawn((
                sprite,
                GridPosition::new(grid),
                Visibility::default(),
                IsoRenderable::with_offset(y_offset, *layer, *depth_offset),
                Transform::from_translation(iso_translation(grid, y_offset, *layer, *depth_offset)),
            ));

            if let CutawayRule::HideWhenPlayerInside { zone } = &prop.cutaway {
                body.insert(CutawayWhenPlayerInside { zone: zone.clone() });
            }
        }
    }
}
