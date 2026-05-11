use bevy::prelude::*;

use crate::gameplay::StationKind;
use crate::universe::{UniverseLayer, WorldMap};

#[derive(Resource)]
pub struct WorldBlueprint {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TilePlacement>,
    pub props: Vec<PropPlacement>,
    pub stations: Vec<StationPlacement>,
    pub seats: Vec<Vec2>,
    pub entrance: Vec2,
}

impl WorldBlueprint {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            tiles: Vec::new(),
            props: Vec::new(),
            stations: Vec::new(),
            seats: Vec::new(),
            entrance: Vec2::ZERO,
        }
    }

    pub fn add_tile(&mut self, tile: TilePlacement) {
        self.tiles.push(tile);
    }

    pub fn add_prop(&mut self, prop: PropPlacement) {
        self.props.push(prop);
    }

    pub fn add_station(&mut self, station: StationPlacement) {
        self.stations.push(station);
    }

    pub fn add_seat(&mut self, seat: Vec2) {
        self.seats.push(seat);
    }

    pub fn set_entrance(&mut self, entrance: Vec2) {
        self.entrance = entrance;
    }

    pub fn build_world_map(&self) -> WorldMap {
        let mut map = WorldMap::new(self.width, self.height);

        for prop in &self.props {
            prop.footprint.block(prop.grid, &mut map);
        }
        for station in &self.stations {
            station.prop.footprint.block(station.prop.grid, &mut map);
        }

        map
    }
}

#[derive(Component)]
pub struct TavernMap {
    pub seats: Vec<Vec2>,
    pub entrance: Vec2,
}

#[derive(Clone, Copy)]
pub struct TilePlacement {
    pub grid: IVec2,
    pub kind: TileKind,
}

impl TilePlacement {
    pub fn new(grid: IVec2, kind: TileKind) -> Self {
        Self { grid, kind }
    }

    pub fn grid_vec(self) -> Vec2 {
        Vec2::new(self.grid.x as f32, self.grid.y as f32)
    }
}

#[derive(Clone, Copy)]
pub enum TileKind {
    Ground,
    TavernFloor,
    Porch,
}

impl TileKind {
    pub fn asset_path(self) -> &'static str {
        match self {
            TileKind::Ground => "tiles/wasteland_ground.png",
            TileKind::TavernFloor => "tiles/tavern_floor.png",
            TileKind::Porch => "tiles/porch_wood.png",
        }
    }

    pub fn layer(self) -> UniverseLayer {
        match self {
            TileKind::Ground => UniverseLayer::Ground,
            TileKind::TavernFloor | TileKind::Porch => UniverseLayer::Floor,
        }
    }
}

pub struct StationPlacement {
    pub kind: StationKind,
    pub prop: PropPlacement,
}

impl StationPlacement {
    pub fn new(kind: StationKind, prop: PropPlacement) -> Self {
        Self { kind, prop }
    }
}

pub struct PropPlacement {
    pub grid: Vec2,
    pub visual: PropVisual,
    pub footprint: Footprint,
}

impl PropPlacement {
    pub fn prop(
        grid: Vec2,
        width: f32,
        height: f32,
        color: Color,
        depth_offset: f32,
        footprint: Footprint,
    ) -> Self {
        Self {
            grid,
            visual: PropVisual::MeshRect {
                size: Vec2::new(width, height),
                color,
                y_offset: height * 0.45,
                layer: UniverseLayer::Prop,
                depth_offset,
                shadow: Some(ShadowSpec {
                    size: Vec2::new(width * 0.95, 13.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.34),
                    y_offset: 3.0,
                    depth_offset: depth_offset - 1.0,
                }),
            },
            footprint,
        }
    }

    pub fn wall(grid: Vec2, width: f32, height: f32, color: Color, y_offset: f32) -> Self {
        Self {
            grid,
            visual: PropVisual::MeshRect {
                size: Vec2::new(width, height),
                color,
                y_offset,
                layer: UniverseLayer::Wall,
                depth_offset: 0.0,
                shadow: None,
            },
            footprint: Footprint::Cells(vec![IVec2::new(
                grid.x.round() as i32,
                grid.y.round() as i32,
            )]),
        }
    }

    pub fn with_depth_offset(mut self, depth_offset: f32) -> Self {
        let PropVisual::MeshRect {
            depth_offset: visual_depth,
            ..
        } = &mut self.visual;
        *visual_depth = depth_offset;
        self
    }
}

pub enum PropVisual {
    MeshRect {
        size: Vec2,
        color: Color,
        y_offset: f32,
        layer: UniverseLayer,
        depth_offset: f32,
        shadow: Option<ShadowSpec>,
    },
}

pub struct ShadowSpec {
    pub size: Vec2,
    pub color: Color,
    pub y_offset: f32,
    pub depth_offset: f32,
}

pub enum Footprint {
    None,
    Nearest,
    Cells(Vec<IVec2>),
}

impl Footprint {
    fn block(&self, grid: Vec2, map: &mut WorldMap) {
        match self {
            Footprint::None => {}
            Footprint::Nearest => map.block_nearest(grid),
            Footprint::Cells(cells) => {
                for cell in cells {
                    map.set_blocked(cell.x, cell.y, true);
                }
            }
        }
    }
}
