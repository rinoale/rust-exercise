use bevy::prelude::*;

use crate::config::{
    MAP_HEIGHT, MAP_WIDTH, TAVERN_MAX_X, TAVERN_MAX_Y, TAVERN_MIN_X, TAVERN_MIN_Y, TILE_WIDTH,
};
use crate::gameplay::StationKind;
use crate::rendering::RenderAnchor;
use crate::universe::UniverseLayer;
use crate::world::blueprint::{
    CutawayRule, Footprint, PropPlacement, PropVisual, StationPlacement, TileKind, TilePlacement,
    WorldBlueprint, WorldZone,
};

pub fn demo_tavern_blueprint() -> WorldBlueprint {
    let mut blueprint = WorldBlueprint::new(MAP_WIDTH, MAP_HEIGHT);

    add_tiles(&mut blueprint);
    add_tavern_shell(&mut blueprint);
    add_furniture(&mut blueprint);
    add_markers(&mut blueprint);

    blueprint
}

fn add_tiles(blueprint: &mut WorldBlueprint) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            blueprint.add_tile(TilePlacement::new(IVec2::new(x, y), tile_kind_for(x, y)));
        }
    }
}

fn tile_kind_for(x: i32, y: i32) -> TileKind {
    if is_tavern_floor(x, y) {
        TileKind::TavernFloor
    } else if is_tavern_porch(x, y) {
        TileKind::Porch
    } else {
        TileKind::Ground
    }
}

fn is_tavern_floor(x: i32, y: i32) -> bool {
    (TAVERN_MIN_X..=TAVERN_MAX_X).contains(&x) && (TAVERN_MIN_Y..=TAVERN_MAX_Y).contains(&y)
}

fn is_tavern_porch(x: i32, y: i32) -> bool {
    (TAVERN_MIN_X - 1..=TAVERN_MAX_X + 1).contains(&x)
        && (TAVERN_MAX_Y..=TAVERN_MAX_Y + 2).contains(&y)
}

fn add_tavern_shell(blueprint: &mut WorldBlueprint) {
    let wall = Color::srgb(0.20, 0.15, 0.11);
    let highlight = Color::srgb(0.34, 0.26, 0.18);

    blueprint.add_prop(PropPlacement {
        grid: Vec2::new(
            (TAVERN_MIN_X + TAVERN_MAX_X) as f32 * 0.5,
            (TAVERN_MIN_Y + TAVERN_MAX_Y) as f32 * 0.5,
        ),
        visual: PropVisual::MeshRect {
            size: Vec2::new(760.0, 280.0),
            color: Color::srgb(0.10, 0.08, 0.055),
            anchor: RenderAnchor::CustomYOffset(92.0),
            layer: UniverseLayer::Roof,
            depth_offset: 80.0,
            shadow: None,
        },
        footprint: Footprint::None,
        cutaway: CutawayRule::HideWhenPlayerInside {
            zone: "main_tavern".to_string(),
        },
    });

    for x in TAVERN_MIN_X..=TAVERN_MAX_X {
        blueprint.add_prop(PropPlacement::wall(
            Vec2::new(x as f32, TAVERN_MIN_Y as f32),
            TILE_WIDTH * 0.9,
            58.0,
            wall,
            34.0,
        ));
    }

    for y in TAVERN_MIN_Y + 1..=TAVERN_MAX_Y {
        blueprint.add_prop(
            PropPlacement::wall(
                Vec2::new(TAVERN_MIN_X as f32, y as f32),
                TILE_WIDTH * 0.9,
                58.0,
                highlight,
                30.0,
            )
            .with_depth_offset(-1.0),
        );
    }

    for x in [8.0, 12.0, 16.0] {
        blueprint.add_prop(PropPlacement::prop(
            Vec2::new(x, 4.2),
            38.0,
            26.0,
            Color::srgb(0.08, 0.075, 0.06),
            35.0,
            Footprint::None,
        ));
    }

    blueprint.add_prop(PropPlacement::prop(
        Vec2::new(20.2, 9.2),
        44.0,
        38.0,
        Color::srgb(0.26, 0.16, 0.10),
        33.0,
        Footprint::None,
    ));
}

fn add_furniture(blueprint: &mut WorldBlueprint) {
    let wood = Color::srgb(0.34, 0.21, 0.11);
    let dark_wood = Color::srgb(0.20, 0.12, 0.07);
    let rust = Color::srgb(0.47, 0.20, 0.11);
    let canvas = Color::srgb(0.17, 0.22, 0.18);

    for x in 8..=15 {
        blueprint.add_prop(PropPlacement::prop(
            Vec2::new(x as f32, 6.6),
            48.0,
            28.0,
            wood,
            42.0,
            Footprint::Nearest,
        ));
    }

    blueprint.add_station(StationPlacement::new(
        StationKind::Stove,
        PropPlacement::prop(
            Vec2::new(8.1, 6.0),
            46.0,
            46.0,
            rust,
            45.0,
            Footprint::Nearest,
        ),
    ));
    blueprint.add_station(StationPlacement::new(
        StationKind::Pantry,
        PropPlacement::prop(
            Vec2::new(6.4, 12.2),
            44.0,
            42.0,
            dark_wood,
            45.0,
            Footprint::Nearest,
        ),
    ));
    blueprint.add_station(StationPlacement::new(
        StationKind::WaterStill,
        PropPlacement::prop(
            Vec2::new(9.2, 12.5),
            42.0,
            46.0,
            Color::srgb(0.22, 0.31, 0.34),
            45.0,
            Footprint::Nearest,
        ),
    ));
    blueprint.add_station(StationPlacement::new(
        StationKind::Radio,
        PropPlacement::prop(
            Vec2::new(18.2, 6.3),
            34.0,
            36.0,
            Color::srgb(0.12, 0.16, 0.13),
            45.0,
            Footprint::Nearest,
        ),
    ));

    for table in [
        Vec2::new(11.0, 9.4),
        Vec2::new(13.2, 9.4),
        Vec2::new(15.4, 9.4),
        Vec2::new(12.2, 11.6),
        Vec2::new(14.5, 11.6),
        Vec2::new(16.7, 11.6),
    ] {
        blueprint.add_prop(PropPlacement::prop(
            table,
            42.0,
            20.0,
            wood,
            36.0,
            Footprint::Nearest,
        ));
        blueprint.add_prop(PropPlacement::prop(
            table + Vec2::new(-0.35, 0.35),
            22.0,
            16.0,
            dark_wood,
            35.0,
            Footprint::None,
        ));
    }

    for barricade in [
        Vec2::new(21.0, 5.0),
        Vec2::new(22.0, 6.0),
        Vec2::new(3.0, 15.0),
        Vec2::new(4.0, 16.0),
    ] {
        blueprint.add_prop(PropPlacement::prop(
            barricade,
            54.0,
            22.0,
            canvas,
            25.0,
            Footprint::Nearest,
        ));
    }
}

fn add_markers(blueprint: &mut WorldBlueprint) {
    blueprint.add_zone(WorldZone::new(
        "main_tavern",
        Vec2::new(TAVERN_MIN_X as f32, TAVERN_MIN_Y as f32),
        Vec2::new(TAVERN_MAX_X as f32, TAVERN_MAX_Y as f32),
    ));

    for seat in [
        Vec2::new(11.2, 8.9),
        Vec2::new(13.0, 8.8),
        Vec2::new(15.1, 9.1),
        Vec2::new(12.0, 11.2),
        Vec2::new(14.1, 11.0),
        Vec2::new(16.4, 11.2),
    ] {
        blueprint.add_seat(seat);
    }

    blueprint.set_entrance(Vec2::new(20.8, 9.0));
}
