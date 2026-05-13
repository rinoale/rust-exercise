use bevy::prelude::*;

use crate::gameplay::StationKind;
use crate::rendering::RenderAnchor;
use crate::universe::{EdgeDirection, UniverseLayer, WorldMap};

#[derive(Resource)]
pub struct WorldBlueprint {
    pub width: i32,
    pub height: i32,
    pub levels: i32,
    pub tiles: Vec<TilePlacement>,
    pub collision_cells: Vec<CollisionCell>,
    pub props: Vec<PropPlacement>,
    pub edge_walls: Vec<EdgeWallPlacement>,
    pub stations: Vec<StationPlacement>,
    pub seats: Vec<NavPoint>,
    pub entrance: NavPoint,
    pub vertical_links: Vec<VerticalLink>,
    pub zones: Vec<WorldZone>,
}

impl WorldBlueprint {
    pub fn new(width: i32, height: i32) -> Self {
        Self::with_levels(width, height, 1)
    }

    pub fn with_levels(width: i32, height: i32, levels: i32) -> Self {
        Self {
            width,
            height,
            levels: levels.max(1),
            tiles: Vec::new(),
            collision_cells: Vec::new(),
            props: Vec::new(),
            edge_walls: Vec::new(),
            stations: Vec::new(),
            seats: Vec::new(),
            entrance: NavPoint::new(Vec2::ZERO),
            vertical_links: Vec::new(),
            zones: Vec::new(),
        }
    }

    pub fn add_tile(&mut self, tile: TilePlacement) {
        self.tiles.push(tile);
    }

    pub fn add_collision_cell(&mut self, cell: CollisionCell) {
        self.collision_cells.push(cell);
    }

    pub fn add_prop(&mut self, prop: PropPlacement) {
        self.props.push(prop);
    }

    pub fn add_edge_wall(&mut self, wall: EdgeWallPlacement) {
        self.edge_walls.push(wall);
    }

    pub fn add_station(&mut self, station: StationPlacement) {
        self.stations.push(station);
    }

    pub fn add_seat(&mut self, seat: NavPoint) {
        self.seats.push(seat);
    }

    pub fn set_entrance(&mut self, entrance: NavPoint) {
        self.entrance = entrance;
    }

    pub fn add_vertical_link(&mut self, link: VerticalLink) {
        self.vertical_links.push(link);
    }

    pub fn add_zone(&mut self, zone: WorldZone) {
        self.zones.push(zone);
    }

    pub fn zone_contains_at(&self, zone_name: &str, grid: Vec2, z: i32) -> bool {
        self.zones
            .iter()
            .any(|zone| zone.name == zone_name && zone.contains_at(grid, z))
    }

    pub fn build_world_map(&self) -> WorldMap {
        let mut map = WorldMap::with_levels(self.width, self.height, self.levels);

        for prop in &self.props {
            prop.footprint.block(prop.grid, prop.z, &mut map);
        }
        for wall in &self.edge_walls {
            map.set_edge_wall_at(wall.grid.x, wall.grid.y, wall.z, wall.direction, true);
            wall.prop.footprint.block(wall.prop.grid, wall.z, &mut map);
        }
        for station in &self.stations {
            station
                .prop
                .footprint
                .block(station.prop.grid, station.prop.z, &mut map);
        }
        for cell in &self.collision_cells {
            map.set_blocked_at(cell.grid.x, cell.grid.y, cell.z, cell.blocked);
        }

        map
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavPoint {
    pub value: Vec2,
    pub z: i32,
}

impl NavPoint {
    pub fn new(value: Vec2) -> Self {
        Self::at(value, 0)
    }

    pub fn at(value: Vec2, z: i32) -> Self {
        Self { value, z }
    }

    pub fn offset(self, offset: Vec2) -> Self {
        Self {
            value: self.value + offset,
            z: self.z,
        }
    }
}

#[derive(Clone)]
pub struct VerticalLink {
    pub id: String,
    pub kind: VerticalLinkKind,
    pub lower: NavPoint,
    pub upper: NavPoint,
    pub bidirectional: bool,
}

impl VerticalLink {
    pub fn new(
        id: impl Into<String>,
        kind: VerticalLinkKind,
        lower: NavPoint,
        upper: NavPoint,
        bidirectional: bool,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            lower,
            upper,
            bidirectional,
        }
    }

    pub fn destination_from_near(&self, grid: Vec2, z: i32, radius: f32) -> Option<NavPoint> {
        if z == self.lower.z && grid.distance(self.lower.value) <= radius {
            return Some(self.upper);
        }
        if self.bidirectional && z == self.upper.z && grid.distance(self.upper.value) <= radius {
            return Some(self.lower);
        }
        None
    }

    pub fn entry_for_level(&self, z: i32) -> Option<NavPoint> {
        if z == self.lower.z {
            Some(self.lower)
        } else if self.bidirectional && z == self.upper.z {
            Some(self.upper)
        } else {
            None
        }
    }

    pub fn exit_from_entry(&self, entry: NavPoint) -> Option<NavPoint> {
        if entry.z == self.lower.z && entry.value.distance(self.lower.value) <= 0.01 {
            Some(self.upper)
        } else if self.bidirectional
            && entry.z == self.upper.z
            && entry.value.distance(self.upper.value) <= 0.01
        {
            Some(self.lower)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalLinkKind {
    Stairs,
    Ladder,
}

impl VerticalLinkKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Stairs => "stairs",
            Self::Ladder => "ladder",
        }
    }
}

pub struct WorldZone {
    pub name: String,
    pub min: Vec2,
    pub max: Vec2,
    pub min_z: i32,
    pub max_z: i32,
}

impl WorldZone {
    pub fn new(name: impl Into<String>, min: Vec2, max: Vec2) -> Self {
        Self::with_levels(name, min, max, 0, 0)
    }

    pub fn with_levels(
        name: impl Into<String>,
        min: Vec2,
        max: Vec2,
        min_z: i32,
        max_z: i32,
    ) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            min_z,
            max_z,
        }
    }

    #[cfg(test)]
    pub fn contains(&self, grid: Vec2) -> bool {
        self.contains_at(grid, 0)
    }

    pub fn contains_at(&self, grid: Vec2, z: i32) -> bool {
        (self.min.x..=self.max.x).contains(&grid.x)
            && (self.min.y..=self.max.y).contains(&grid.y)
            && (self.min_z..=self.max_z).contains(&z)
    }
}

#[derive(Component)]
pub struct TavernMap {
    pub seats: Vec<NavPoint>,
    pub entrance: NavPoint,
    pub vertical_links: Vec<VerticalLink>,
}

#[derive(Clone, Copy)]
pub struct CollisionCell {
    pub grid: IVec2,
    pub z: i32,
    pub blocked: bool,
}

impl CollisionCell {
    pub fn new(grid: IVec2, z: i32, blocked: bool) -> Self {
        Self { grid, z, blocked }
    }
}

#[derive(Clone, Copy)]
pub struct TilePlacement {
    pub grid: IVec2,
    pub z: i32,
    pub kind: TileKind,
}

impl TilePlacement {
    pub fn new(grid: IVec2, kind: TileKind) -> Self {
        Self::at(grid, 0, kind)
    }

    pub fn at(grid: IVec2, z: i32, kind: TileKind) -> Self {
        Self { grid, z, kind }
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
    pub fn asset_id(self, grid: IVec2) -> &'static str {
        match self {
            TileKind::Ground => match (grid.x + grid.y).rem_euclid(3) {
                0 => "tile.wasteland.dirt_a",
                1 => "tile.wasteland.dirt_b",
                _ => "tile.wasteland.scrub_a",
            },
            TileKind::TavernFloor => match (grid.x * 3 + grid.y).rem_euclid(3) {
                0 => "tile.tavern.floor_worn_a",
                1 => "tile.tavern.floor_worn_b",
                _ => "tile.tavern.floor_worn_c",
            },
            TileKind::Porch => "tile.porch.planks_worn_a",
        }
    }

    pub fn fallback_asset_path(self, grid: IVec2) -> &'static str {
        match self {
            TileKind::Ground => match (grid.x + grid.y).rem_euclid(3) {
                0 => "tiles/wasteland_dirt_a.png",
                1 => "tiles/wasteland_dirt_b.png",
                _ => "tiles/wasteland_scrub_a.png",
            },
            TileKind::TavernFloor => match (grid.x * 3 + grid.y).rem_euclid(3) {
                0 => "tiles/tavern_floor_worn_a.png",
                1 => "tiles/tavern_floor_worn_b.png",
                _ => "tiles/tavern_floor_worn_c.png",
            },
            TileKind::Porch => "tiles/porch_planks_worn_a.png",
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

pub struct EdgeWallPlacement {
    pub grid: IVec2,
    pub z: i32,
    pub direction: EdgeDirection,
    pub prop: PropPlacement,
}

impl EdgeWallPlacement {
    pub fn new(grid: IVec2, z: i32, direction: EdgeDirection, prop: PropPlacement) -> Self {
        Self {
            grid,
            z,
            direction,
            prop,
        }
    }
}

pub struct PropPlacement {
    pub instance_id: Option<String>,
    pub prefab_id: Option<String>,
    pub grid: Vec2,
    pub z: i32,
    pub visual: PropVisual,
    pub footprint: Footprint,
    pub cutaway: CutawayRule,
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
            instance_id: None,
            prefab_id: None,
            grid,
            z: 0,
            visual: PropVisual::MeshRect {
                size: Vec2::new(width, height),
                color,
                anchor: RenderAnchor::LowProp,
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
            cutaway: CutawayRule::None,
        }
    }

    pub fn wall(grid: Vec2, width: f32, height: f32, color: Color) -> Self {
        Self {
            instance_id: None,
            prefab_id: None,
            grid,
            z: 0,
            visual: PropVisual::MeshRect {
                size: Vec2::new(width, height),
                color,
                anchor: RenderAnchor::WallBaseline,
                layer: UniverseLayer::Wall,
                depth_offset: 0.0,
                shadow: None,
            },
            footprint: Footprint::Cells(vec![IVec2::new(
                grid.x.round() as i32,
                grid.y.round() as i32,
            )]),
            cutaway: CutawayRule::None,
        }
    }

    pub fn with_depth_offset(mut self, depth_offset: f32) -> Self {
        match &mut self.visual {
            PropVisual::MeshRect {
                depth_offset: visual_depth,
                ..
            }
            | PropVisual::Sprite {
                depth_offset: visual_depth,
                ..
            } => *visual_depth = depth_offset,
        }
        self
    }

    pub fn with_source(mut self, instance_id: String, prefab_id: String) -> Self {
        self.instance_id = Some(instance_id);
        self.prefab_id = Some(prefab_id);
        self
    }
}

#[derive(Clone)]
pub enum CutawayRule {
    None,
    HideWhenPlayerInside { zone: String },
}

pub enum PropVisual {
    MeshRect {
        size: Vec2,
        color: Color,
        anchor: RenderAnchor,
        layer: UniverseLayer,
        depth_offset: f32,
        shadow: Option<ShadowSpec>,
    },
    Sprite {
        asset_id: Option<String>,
        asset_path: String,
        size: Vec2,
        anchor: RenderAnchor,
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

#[derive(Component)]
pub struct WorldSceneEntity;

#[derive(Component)]
pub struct WorldObjectEntity {
    pub instance_id: String,
}

impl Footprint {
    fn block(&self, grid: Vec2, z: i32, map: &mut WorldMap) {
        match self {
            Footprint::None => {}
            Footprint::Nearest => map.block_nearest_at(grid, z),
            Footprint::Cells(cells) => {
                for cell in cells {
                    map.set_blocked_at(cell.x, cell.y, z, true);
                }
            }
        }
    }
}
