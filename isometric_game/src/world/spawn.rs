use bevy::prelude::*;

use crate::gameplay::{Station, StationCooldown};
use crate::rendering::{AssetRegistry, create_diamond_mesh, create_rect_mesh, tile_sprite};
use crate::universe::{GridPosition, IsoRenderable, UniverseLayer, iso_translation_at};
use crate::world::blueprint::{
    CutawayRule, PropPlacement, PropVisual, TavernMap, WorldBlueprint, WorldObjectEntity,
    WorldSceneEntity,
};
use crate::world::cutaway::CutawayWhenPlayerInside;

pub fn spawn_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    asset_registry: Res<AssetRegistry>,
    blueprint: Res<WorldBlueprint>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_world_entities(
        &mut commands,
        &asset_server,
        &asset_registry,
        &blueprint,
        &mut meshes,
        &mut materials,
    );
}

pub fn spawn_world_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    asset_registry: &AssetRegistry,
    blueprint: &WorldBlueprint,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    for tile in &blueprint.tiles {
        let grid = tile.grid_vec();
        let layer = tile.kind.layer();
        let asset_path = asset_registry.sprite_path_or_fallback(
            tile.kind.asset_id(tile.grid),
            tile.kind.fallback_asset_path(tile.grid),
        );
        commands.spawn((
            WorldSceneEntity,
            tile_sprite(asset_server.load(asset_path.to_string())),
            GridPosition::at(grid, tile.z),
            Visibility::default(),
            IsoRenderable::new(0.0, layer),
            Transform::from_translation(iso_translation_at(grid, tile.z, 0.0, layer, 0.0)),
        ));
    }

    for prop in &blueprint.props {
        spawn_prop(commands, asset_server, meshes, materials, prop);
    }

    for wall in &blueprint.edge_walls {
        spawn_prop(commands, asset_server, meshes, materials, &wall.prop);
    }

    for station in &blueprint.stations {
        spawn_prop(commands, asset_server, meshes, materials, &station.prop);
        commands.spawn((
            WorldSceneEntity,
            Station { kind: station.kind },
            StationCooldown::default(),
            GridPosition::at(station.prop.grid, station.prop.z),
        ));
    }

    commands.spawn((
        WorldSceneEntity,
        TavernMap {
            seats: blueprint.seats.clone(),
            entrance: blueprint.entrance,
            vertical_links: blueprint.vertical_links.clone(),
        },
    ));
}

fn spawn_prop(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
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
                    WorldSceneEntity,
                    Mesh2d(meshes.add(create_diamond_mesh(shadow.size.x, shadow.size.y))),
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(shadow.color))),
                    GridPosition::at(grid, prop.z),
                    Visibility::default(),
                    IsoRenderable::with_offset(
                        shadow.y_offset,
                        UniverseLayer::Decal,
                        shadow.depth_offset,
                    ),
                    Transform::from_translation(iso_translation_at(
                        grid,
                        prop.z,
                        shadow.y_offset,
                        UniverseLayer::Decal,
                        shadow.depth_offset,
                    )),
                ));
            }

            let mut body = commands.spawn((
                WorldSceneEntity,
                Mesh2d(meshes.add(create_rect_mesh(size.x, size.y))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(*color))),
                GridPosition::at(grid, prop.z),
                Visibility::default(),
                IsoRenderable::with_offset(y_offset, *layer, *depth_offset),
                Transform::from_translation(iso_translation_at(
                    grid,
                    prop.z,
                    y_offset,
                    *layer,
                    *depth_offset,
                )),
            ));

            if let Some(instance_id) = &prop.instance_id {
                body.insert(WorldObjectEntity {
                    instance_id: instance_id.clone(),
                });
            }

            if let CutawayRule::HideWhenPlayerInside { zone } = &prop.cutaway {
                body.insert(CutawayWhenPlayerInside { zone: zone.clone() });
            }
        }
        PropVisual::Sprite {
            asset_id,
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
                    WorldSceneEntity,
                    Mesh2d(meshes.add(create_diamond_mesh(shadow.size.x, shadow.size.y))),
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(shadow.color))),
                    GridPosition::at(grid, prop.z),
                    Visibility::default(),
                    IsoRenderable::with_offset(
                        shadow.y_offset,
                        UniverseLayer::Decal,
                        shadow.depth_offset,
                    ),
                    Transform::from_translation(iso_translation_at(
                        grid,
                        prop.z,
                        shadow.y_offset,
                        UniverseLayer::Decal,
                        shadow.depth_offset,
                    )),
                ));
            }

            let mut sprite = Sprite::from_image(asset_server.load(asset_path.clone()));
            sprite.custom_size = Some(*size);
            let sprite_name = asset_id.as_deref().unwrap_or(asset_path.as_str());
            let mut body = commands.spawn((
                WorldSceneEntity,
                Name::new(format!("world.sprite.{sprite_name}")),
                sprite,
                GridPosition::at(grid, prop.z),
                Visibility::default(),
                IsoRenderable::with_offset(y_offset, *layer, *depth_offset),
                Transform::from_translation(iso_translation_at(
                    grid,
                    prop.z,
                    y_offset,
                    *layer,
                    *depth_offset,
                )),
            ));

            if let Some(instance_id) = &prop.instance_id {
                body.insert(WorldObjectEntity {
                    instance_id: instance_id.clone(),
                });
            }

            if let CutawayRule::HideWhenPlayerInside { zone } = &prop.cutaway {
                body.insert(CutawayWhenPlayerInside { zone: zone.clone() });
            }
        }
    }
}
