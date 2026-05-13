use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::StationKind;
use crate::rendering::{AssetRegistry, RenderAnchor};
use crate::universe::{EdgeDirection, UniverseLayer};
use crate::world::blueprint::{
    CollisionCell, CutawayRule, EdgeWallPlacement, Footprint, NavPoint, PropPlacement, PropVisual,
    ShadowSpec, StationPlacement, TileKind, TilePlacement, VerticalLink, VerticalLinkKind,
    WorldBlueprint, WorldZone,
};
use crate::world::prefabs::{PrefabCatalog, PrefabInstanceFile};

pub const DEMO_TAVERN_PATH: &str = "data/worlds/demo_tavern.ron";

pub fn load_world_document(path: &str) -> Result<WorldDocument, String> {
    let path = project_path(path);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

pub fn save_world_document(path: &str, document: &WorldDocument) -> Result<(), String> {
    let path = project_path(path);
    let pretty = ron::ser::PrettyConfig::new()
        .depth_limit(4)
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let serialized = ron::ser::to_string_pretty(document, pretty)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(&path, serialized)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

pub fn project_path(path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct WorldDocument {
    pub width: i32,
    pub height: i32,
    #[serde(default = "default_world_levels")]
    pub levels: i32,
    #[serde(default)]
    pub prop_styles: Vec<PropStyleDefinitionFile>,
    pub tile_regions: Vec<TileRegionFile>,
    #[serde(default)]
    pub collision_cells: Vec<CollisionCellFile>,
    #[serde(default)]
    pub zones: Vec<ZoneFile>,
    #[serde(default)]
    pub props: Vec<PropFile>,
    #[serde(default)]
    pub prop_runs: Vec<PropRunFile>,
    #[serde(default)]
    pub edge_walls: Vec<EdgeWallFile>,
    #[serde(default)]
    pub stations: Vec<StationFile>,
    #[serde(default)]
    pub vertical_links: Vec<VerticalLinkFile>,
    #[serde(default)]
    pub prefab_instances: Vec<PrefabInstanceFile>,
    #[serde(default)]
    pub seats: Vec<NavPointFile>,
    pub entrance: NavPointFile,
}

impl WorldDocument {
    pub fn empty_scene(width: i32, height: i32, levels: i32) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            levels: levels.max(1),
            prop_styles: Vec::new(),
            tile_regions: Vec::new(),
            collision_cells: Vec::new(),
            zones: Vec::new(),
            props: Vec::new(),
            prop_runs: Vec::new(),
            edge_walls: Vec::new(),
            stations: Vec::new(),
            vertical_links: Vec::new(),
            prefab_instances: Vec::new(),
            seats: Vec::new(),
            entrance: NavPointFile::Grid(vec![0.0, 0.0]),
        }
    }

    pub fn empty_demo() -> Self {
        Self {
            width: 26,
            height: 18,
            levels: 1,
            prop_styles: Vec::new(),
            tile_regions: vec![TileRegionFile {
                kind: TileKindFile::Ground,
                z: 0,
                from: vec![0, 0],
                to: vec![25, 17],
            }],
            collision_cells: Vec::new(),
            zones: Vec::new(),
            props: Vec::new(),
            prop_runs: Vec::new(),
            edge_walls: Vec::new(),
            stations: Vec::new(),
            vertical_links: Vec::new(),
            prefab_instances: Vec::new(),
            seats: Vec::new(),
            entrance: NavPointFile::Grid(vec![18.0, 10.0]),
        }
    }

    pub fn next_prefab_instance_id(&self, prefab_id: &str) -> String {
        let base = prefab_id.replace('.', "_");
        let mut index = self.prefab_instances.len() + 1;
        loop {
            let candidate = format!("{base}_{index:03}");
            if !self
                .prefab_instances
                .iter()
                .any(|instance| instance.id == candidate)
            {
                return candidate;
            }
            index += 1;
        }
    }

    pub fn into_blueprint(
        self,
        asset_registry: &AssetRegistry,
        prefab_catalog: &PrefabCatalog,
    ) -> Result<WorldBlueprint, String> {
        if self.width <= 0 || self.height <= 0 {
            return Err(format!(
                "world dimensions must be positive, got {}x{}",
                self.width, self.height
            ));
        }
        let levels = self.levels;
        if levels <= 0 {
            return Err(format!("world levels must be positive, got {levels}"));
        }

        let styles = StyleLibrary::from_definitions(self.prop_styles)?;
        let mut blueprint = WorldBlueprint::with_levels(self.width, self.height, levels);
        add_tiles(
            &mut blueprint,
            self.width,
            self.height,
            levels,
            &self.tile_regions,
        )?;

        for cell in self.collision_cells {
            validate_z(cell.z, levels, "collision cell")?;
            blueprint.add_collision_cell(cell.into_collision_cell()?);
        }
        for zone in self.zones {
            validate_z_range(zone.min_z, zone.max_z, levels, "zone")?;
            blueprint.add_zone(zone.into_zone()?);
        }
        for prop in self.props {
            validate_z(prop.z, levels, "prop")?;
            blueprint.add_prop(prop.into_prop(&styles, asset_registry)?);
        }
        for run in self.prop_runs {
            validate_z(run.z, levels, "prop run")?;
            for prop in run.into_props(&styles, asset_registry)? {
                blueprint.add_prop(prop);
            }
        }
        for wall in self.edge_walls {
            validate_z(wall.z, levels, "edge wall")?;
            blueprint.add_edge_wall(wall.into_edge_wall(&styles, asset_registry)?);
        }
        for station in self.stations {
            validate_z(station.z, levels, "station")?;
            blueprint.add_station(station.into_station(&styles, asset_registry)?);
        }
        for link in self.vertical_links {
            blueprint.add_vertical_link(link.into_vertical_link(levels)?);
        }
        for prefab_instance in self.prefab_instances {
            validate_z(prefab_instance.z, levels, "prefab instance")?;
            prefab_catalog.add_instance_to_blueprint(
                prefab_instance,
                asset_registry,
                &mut blueprint,
            )?;
        }
        for seat in self.seats {
            blueprint.add_seat(seat.into_nav_point(levels, "seat marker")?);
        }

        blueprint.set_entrance(self.entrance.into_nav_point(levels, "entrance marker")?);

        Ok(blueprint)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ZoneFile {
    pub name: String,
    pub min: Vec<f32>,
    pub max: Vec<f32>,
    #[serde(default = "default_z")]
    pub min_z: i32,
    #[serde(default = "default_z")]
    pub max_z: i32,
}

impl ZoneFile {
    fn into_zone(self) -> Result<WorldZone, String> {
        let min = vec2(&self.min, "zone min")?;
        let max = vec2(&self.max, "zone max")?;
        if min.x > max.x || min.y > max.y || self.min_z > self.max_z {
            return Err(format!(
                "zone '{}' has invalid bounds {:?}:{} -> {:?}:{}",
                self.name, min, self.min_z, max, self.max_z
            ));
        }

        Ok(WorldZone::with_levels(
            self.name, min, max, self.min_z, self.max_z,
        ))
    }
}

struct StyleLibrary {
    styles: HashMap<String, PropStyleFile>,
}

impl StyleLibrary {
    fn from_definitions(definitions: Vec<PropStyleDefinitionFile>) -> Result<Self, String> {
        let mut styles = HashMap::new();

        for definition in definitions {
            if styles
                .insert(definition.name.clone(), definition.style)
                .is_some()
            {
                return Err(format!("duplicate prop style '{}'", definition.name));
            }
        }

        Ok(Self { styles })
    }

    fn resolve(&self, style: PropStyleRefFile, context: &str) -> Result<PropStyleFile, String> {
        match style {
            PropStyleRefFile::Named(name) => self
                .styles
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("{context} references unknown prop style '{name}'")),
            PropStyleRefFile::Inline(style) => Ok(style),
        }
    }
}

fn add_tiles(
    blueprint: &mut WorldBlueprint,
    width: i32,
    height: i32,
    levels: i32,
    regions: &[TileRegionFile],
) -> Result<(), String> {
    let mut tiles = vec![None; (width * height * levels) as usize];

    for region in regions {
        let from = ivec2(&region.from, "tile region from")?;
        let to = ivec2(&region.to, "tile region to")?;
        if from.x > to.x || from.y > to.y || region.z < 0 || region.z >= levels {
            return Err(format!(
                "invalid tile region {:?}:{} -> {:?}:{}",
                region.from, region.z, region.to, region.z
            ));
        }

        for y in from.y..=to.y {
            for x in from.x..=to.x {
                if x < 0 || y < 0 || x >= width || y >= height {
                    continue;
                }
                tiles[tile_index(width, height, x, y, region.z)] =
                    Some(region.kind.into_tile_kind());
            }
        }
    }

    for z in 0..levels {
        for y in 0..height {
            for x in 0..width {
                if let Some(kind) = tiles[tile_index(width, height, x, y, z)] {
                    blueprint.add_tile(TilePlacement::at(IVec2::new(x, y), z, kind));
                }
            }
        }
    }

    Ok(())
}

fn tile_index(width: i32, height: i32, x: i32, y: i32, z: i32) -> usize {
    ((z * width * height) + (y * width + x)) as usize
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TileRegionFile {
    pub kind: TileKindFile,
    #[serde(default = "default_z")]
    pub z: i32,
    pub from: Vec<i32>,
    pub to: Vec<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TileKindFile {
    Ground,
    TavernFloor,
    Porch,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CollisionCellFile {
    pub grid: Vec<i32>,
    #[serde(default = "default_z")]
    pub z: i32,
    #[serde(default = "default_blocked")]
    pub blocked: bool,
}

impl CollisionCellFile {
    pub(crate) fn at(grid: IVec2, z: i32, blocked: bool) -> Self {
        Self {
            grid: vec![grid.x, grid.y],
            z,
            blocked,
        }
    }

    fn into_collision_cell(self) -> Result<CollisionCell, String> {
        Ok(CollisionCell::new(
            ivec2(&self.grid, "collision cell grid")?,
            self.z,
            self.blocked,
        ))
    }
}

impl TileKindFile {
    pub(crate) const ALL: [Self; 3] = [Self::Ground, Self::TavernFloor, Self::Porch];

    fn into_tile_kind(self) -> TileKind {
        match self {
            TileKindFile::Ground => TileKind::Ground,
            TileKindFile::TavernFloor => TileKind::TavernFloor,
            TileKindFile::Porch => TileKind::Porch,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Ground => "ground",
            Self::TavernFloor => "tavern floor",
            Self::Porch => "porch",
        }
    }

    pub(crate) fn cycle(self, step: isize) -> Self {
        cycle_value(Self::ALL.as_slice(), self, step)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PropFile {
    pub grid: Vec<f32>,
    #[serde(default = "default_z")]
    pub z: i32,
    pub style: PropStyleRefFile,
}

impl PropFile {
    fn into_prop(
        self,
        styles: &StyleLibrary,
        asset_registry: &AssetRegistry,
    ) -> Result<PropPlacement, String> {
        let grid = vec2(&self.grid, "prop grid")?;
        styles
            .resolve(self.style, "prop")?
            .into_prop(grid, self.z, asset_registry)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PropRunFile {
    pub start: Vec<f32>,
    #[serde(default = "default_z")]
    pub z: i32,
    pub count: usize,
    pub step: Vec<f32>,
    pub style: PropStyleRefFile,
}

impl PropRunFile {
    fn into_props(
        self,
        styles: &StyleLibrary,
        asset_registry: &AssetRegistry,
    ) -> Result<Vec<PropPlacement>, String> {
        let mut props = Vec::with_capacity(self.count);
        let start = vec2(&self.start, "prop run start")?;
        let step = vec2(&self.step, "prop run step")?;
        let style = styles.resolve(self.style, "prop run")?;

        for index in 0..self.count {
            props.push(style.clone().into_prop(
                start + step * index as f32,
                self.z,
                asset_registry,
            )?);
        }

        Ok(props)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct EdgeWallFile {
    pub grid: Vec<i32>,
    #[serde(default = "default_z")]
    pub z: i32,
    pub direction: EdgeDirectionFile,
    pub style: PropStyleRefFile,
}

impl EdgeWallFile {
    fn into_edge_wall(
        self,
        styles: &StyleLibrary,
        asset_registry: &AssetRegistry,
    ) -> Result<EdgeWallPlacement, String> {
        let grid = ivec2(&self.grid, "edge wall grid")?;
        let direction = self.direction.into_edge_direction();
        let mut prop = styles.resolve(self.style, "edge wall")?.into_prop(
            edge_wall_visual_grid(grid, direction),
            self.z,
            asset_registry,
        )?;
        prop.footprint = Footprint::None;

        Ok(EdgeWallPlacement::new(grid, self.z, direction, prop))
    }
}

fn edge_wall_visual_grid(grid: IVec2, direction: EdgeDirection) -> Vec2 {
    let center = Vec2::new(grid.x as f32, grid.y as f32);
    center
        + match direction {
            EdgeDirection::N => Vec2::new(-0.5, 0.0),
            EdgeDirection::E => Vec2::new(0.0, -0.5),
            EdgeDirection::S => Vec2::new(0.5, 0.0),
            EdgeDirection::W => Vec2::new(0.0, 0.5),
        }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum EdgeDirectionFile {
    N,
    E,
    S,
    W,
}

impl EdgeDirectionFile {
    pub(crate) const ALL: [Self; 4] = [Self::N, Self::E, Self::S, Self::W];

    fn into_edge_direction(self) -> EdgeDirection {
        match self {
            Self::N => EdgeDirection::N,
            Self::E => EdgeDirection::E,
            Self::S => EdgeDirection::S,
            Self::W => EdgeDirection::W,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::N => "N",
            Self::E => "E",
            Self::S => "S",
            Self::W => "W",
        }
    }

    pub(crate) fn cycle(self, step: isize) -> Self {
        cycle_value(Self::ALL.as_slice(), self, step)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PropStyleDefinitionFile {
    pub name: String,
    pub style: PropStyleFile,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum PropStyleRefFile {
    Named(String),
    Inline(PropStyleFile),
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PropStyleFile {
    pub size: Vec<f32>,
    pub layer: PropLayerFile,
    #[serde(default)]
    pub asset_id: String,
    #[serde(default)]
    pub asset: String,
    #[serde(default)]
    pub color: Vec<f32>,
    #[serde(default)]
    pub anchor: RenderAnchorFile,
    #[serde(default)]
    pub y_offset: Option<f32>,
    #[serde(default)]
    pub depth_offset: f32,
    #[serde(default)]
    pub shadow: bool,
    #[serde(default)]
    pub footprint: FootprintFile,
    #[serde(default)]
    pub cutaway: CutawayFile,
}

impl PropStyleFile {
    pub(crate) fn into_prop(
        self,
        grid: Vec2,
        z: i32,
        asset_registry: &AssetRegistry,
    ) -> Result<PropPlacement, String> {
        let size = vec2(&self.size, "prop size")?;
        if size.x <= 0.0 || size.y <= 0.0 {
            return Err(format!("prop at {grid:?} has invalid size {size:?}"));
        }

        let layer = self.layer.into_universe_layer();
        let anchor = self
            .y_offset
            .map(RenderAnchor::CustomYOffset)
            .unwrap_or_else(|| self.anchor.into_render_anchor(layer));
        let shadow = self.shadow.then_some(ShadowSpec {
            size: Vec2::new(size.x * 0.95, 13.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.34),
            y_offset: 3.0,
            depth_offset: self.depth_offset - 1.0,
        });

        let visual = if !self.asset_id.is_empty() {
            let asset_path = asset_registry
                .sprite_path(&self.asset_id)
                .map_err(|error| format!("prop at {grid:?}: {error}"))?
                .to_string();
            PropVisual::Sprite {
                asset_id: Some(self.asset_id),
                asset_path,
                size,
                anchor,
                layer,
                depth_offset: self.depth_offset,
                shadow,
            }
        } else if !self.asset.is_empty() {
            PropVisual::Sprite {
                asset_id: None,
                asset_path: self.asset,
                size,
                anchor,
                layer,
                depth_offset: self.depth_offset,
                shadow,
            }
        } else {
            if self.color.is_empty() {
                return Err(format!("prop at {grid:?} needs asset_id, asset, or color"));
            }

            PropVisual::MeshRect {
                size,
                color: color_value(&self.color, "prop color")?,
                anchor,
                layer,
                depth_offset: self.depth_offset,
                shadow,
            }
        };

        Ok(PropPlacement {
            instance_id: None,
            prefab_id: None,
            grid,
            z,
            visual,
            footprint: self.footprint.into_footprint()?,
            cutaway: self.cutaway.into_cutaway_rule(),
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub(crate) enum RenderAnchorFile {
    TileCenter,
    #[default]
    LowProp,
    WallBaseline,
    RoofCover,
    Foot,
    CustomYOffset(f32),
}

impl RenderAnchorFile {
    fn into_render_anchor(self, layer: UniverseLayer) -> RenderAnchor {
        match (self, layer) {
            (RenderAnchorFile::TileCenter, _) => RenderAnchor::TileCenter,
            (RenderAnchorFile::LowProp, UniverseLayer::Wall) => RenderAnchor::WallBaseline,
            (RenderAnchorFile::LowProp, _) => RenderAnchor::LowProp,
            (RenderAnchorFile::WallBaseline, _) => RenderAnchor::WallBaseline,
            (RenderAnchorFile::RoofCover, _) => RenderAnchor::RoofCover,
            (RenderAnchorFile::Foot, _) => RenderAnchor::Foot,
            (RenderAnchorFile::CustomYOffset(offset), _) => RenderAnchor::CustomYOffset(offset),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) enum PropLayerFile {
    Decal,
    Prop,
    Wall,
    Roof,
    Overlay,
}

impl PropLayerFile {
    fn into_universe_layer(self) -> UniverseLayer {
        match self {
            PropLayerFile::Decal => UniverseLayer::Decal,
            PropLayerFile::Prop => UniverseLayer::Prop,
            PropLayerFile::Wall => UniverseLayer::Wall,
            PropLayerFile::Roof => UniverseLayer::Roof,
            PropLayerFile::Overlay => UniverseLayer::Overlay,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) enum CutawayFile {
    #[default]
    None,
    HideWhenPlayerInside {
        zone: String,
    },
}

impl CutawayFile {
    fn into_cutaway_rule(self) -> CutawayRule {
        match self {
            CutawayFile::None => CutawayRule::None,
            CutawayFile::HideWhenPlayerInside { zone } => {
                CutawayRule::HideWhenPlayerInside { zone }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) enum FootprintFile {
    #[default]
    None,
    Nearest,
    Cells(Vec<Vec<i32>>),
}

impl FootprintFile {
    fn into_footprint(self) -> Result<Footprint, String> {
        match self {
            FootprintFile::None => Ok(Footprint::None),
            FootprintFile::Nearest => Ok(Footprint::Nearest),
            FootprintFile::Cells(cells) => {
                let mut parsed_cells = Vec::with_capacity(cells.len());
                for cell in cells {
                    parsed_cells.push(ivec2(&cell, "footprint cell")?);
                }
                Ok(Footprint::Cells(parsed_cells))
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct StationFile {
    pub kind: StationKindFile,
    pub grid: Vec<f32>,
    #[serde(default = "default_z")]
    pub z: i32,
    pub style: PropStyleRefFile,
}

impl StationFile {
    fn into_station(
        self,
        styles: &StyleLibrary,
        asset_registry: &AssetRegistry,
    ) -> Result<StationPlacement, String> {
        let grid = vec2(&self.grid, "station grid")?;
        Ok(StationPlacement::new(
            self.kind.into_station_kind(),
            styles
                .resolve(self.style, "station")?
                .into_prop(grid, self.z, asset_registry)?,
        ))
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) enum StationKindFile {
    Pantry,
    WaterStill,
    Stove,
    Radio,
}

impl StationKindFile {
    pub(crate) fn into_station_kind(self) -> StationKind {
        match self {
            StationKindFile::Pantry => StationKind::Pantry,
            StationKindFile::WaterStill => StationKind::WaterStill,
            StationKindFile::Stove => StationKind::Stove,
            StationKindFile::Radio => StationKind::Radio,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct VerticalLinkFile {
    pub id: String,
    pub kind: VerticalLinkKindFile,
    pub lower: NavPointFile,
    pub upper: NavPointFile,
    #[serde(default = "default_bidirectional")]
    pub bidirectional: bool,
}

impl VerticalLinkFile {
    fn into_vertical_link(self, levels: i32) -> Result<VerticalLink, String> {
        let lower = self.lower.into_nav_point(levels, "vertical link lower")?;
        let upper = self.upper.into_nav_point(levels, "vertical link upper")?;
        if lower.z == upper.z {
            return Err(format!(
                "vertical link '{}' must connect different z levels",
                self.id
            ));
        }

        Ok(VerticalLink::new(
            self.id,
            self.kind.into_vertical_link_kind(),
            lower,
            upper,
            self.bidirectional,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum VerticalLinkKindFile {
    Stairs,
    Ladder,
}

impl VerticalLinkKindFile {
    pub(crate) const ALL: [Self; 2] = [Self::Stairs, Self::Ladder];

    fn into_vertical_link_kind(self) -> VerticalLinkKind {
        match self {
            Self::Stairs => VerticalLinkKind::Stairs,
            Self::Ladder => VerticalLinkKind::Ladder,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Stairs => "stairs",
            Self::Ladder => "ladder",
        }
    }

    pub(crate) fn cycle(self, step: isize) -> Self {
        cycle_value(Self::ALL.as_slice(), self, step)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum NavPointFile {
    Grid(Vec<f32>),
    Detailed {
        grid: Vec<f32>,
        #[serde(default = "default_z")]
        z: i32,
    },
}

impl NavPointFile {
    pub(crate) fn at(grid: Vec2, z: i32) -> Self {
        Self::Detailed {
            grid: vec![grid.x, grid.y],
            z,
        }
    }

    pub(crate) fn grid_vec(&self) -> Result<Vec2, String> {
        match self {
            Self::Grid(grid) => vec2(grid, "nav point grid"),
            Self::Detailed { grid, .. } => vec2(grid, "nav point grid"),
        }
    }

    pub(crate) fn z(&self) -> i32 {
        match self {
            Self::Grid(_) => 0,
            Self::Detailed { z, .. } => *z,
        }
    }

    pub(crate) fn set(&mut self, grid: Vec2, z: i32) {
        *self = Self::at(grid, z);
    }

    fn into_nav_point(self, levels: i32, label: &str) -> Result<NavPoint, String> {
        let (grid, z) = match self {
            Self::Grid(grid) => (grid, 0),
            Self::Detailed { grid, z } => (grid, z),
        };
        validate_z(z, levels, label)?;
        Ok(NavPoint::at(vec2(&grid, label)?, z))
    }
}

fn ivec2(value: &[i32], label: &str) -> Result<IVec2, String> {
    if value.len() != 2 {
        return Err(format!("{label} must have exactly 2 values, got {value:?}"));
    }

    Ok(IVec2::new(value[0], value[1]))
}

pub(crate) fn vec2(value: &[f32], label: &str) -> Result<Vec2, String> {
    if value.len() != 2 {
        return Err(format!("{label} must have exactly 2 values, got {value:?}"));
    }

    Ok(Vec2::new(value[0], value[1]))
}

fn color_value(value: &[f32], label: &str) -> Result<Color, String> {
    if value.len() != 3 {
        return Err(format!("{label} must have exactly 3 values, got {value:?}"));
    }

    Ok(Color::srgb(value[0], value[1], value[2]))
}

fn validate_z(z: i32, levels: i32, label: &str) -> Result<(), String> {
    if z < 0 || z >= levels {
        return Err(format!(
            "{label} has invalid z {z}; world levels are 0..{}",
            levels - 1
        ));
    }
    Ok(())
}

fn validate_z_range(min_z: i32, max_z: i32, levels: i32, label: &str) -> Result<(), String> {
    validate_z(min_z, levels, label)?;
    validate_z(max_z, levels, label)?;
    if min_z > max_z {
        return Err(format!("{label} has invalid z range {min_z}..{max_z}"));
    }
    Ok(())
}

fn default_world_levels() -> i32 {
    1
}

fn default_z() -> i32 {
    0
}

fn default_bidirectional() -> bool {
    true
}

fn default_blocked() -> bool {
    true
}

fn cycle_value<T: Copy + PartialEq>(values: &[T], current: T, step: isize) -> T {
    let index = values
        .iter()
        .position(|value| *value == current)
        .unwrap_or_default() as isize;
    values[(index + step).rem_euclid(values.len() as isize) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_tavern_file_loads_as_blueprint() {
        let asset_registry =
            crate::rendering::load_asset_registry(crate::rendering::ASSET_REGISTRY_PATH)
                .expect("asset registry should load");
        let prefab_catalog = crate::world::prefabs::load_prefab_catalog(
            crate::world::prefabs::PREFAB_CATALOG_PATH,
            &asset_registry,
        )
        .expect("prefab catalog should load");
        let blueprint = load_world_document(DEMO_TAVERN_PATH)
            .expect("demo tavern document should load")
            .into_blueprint(&asset_registry, &prefab_catalog)
            .expect("demo tavern should build");

        assert_eq!(blueprint.width, 26);
        assert_eq!(blueprint.height, 18);
        assert_eq!(blueprint.levels, 1);
        assert_eq!(blueprint.tiles.len(), 26 * 18);
        assert_eq!(blueprint.zones.len(), 1);
        assert_eq!(blueprint.collision_cells.len(), 0);
        assert_eq!(blueprint.stations.len(), 4);
        assert_eq!(blueprint.seats.len(), 6);
        assert_eq!(blueprint.props.len(), 64);
        assert_eq!(
            blueprint
                .props
                .iter()
                .filter(|prop| matches!(
                    (&prop.visual, &prop.cutaway),
                    (
                        PropVisual::Sprite {
                            layer: UniverseLayer::Roof,
                            ..
                        },
                        CutawayRule::HideWhenPlayerInside { zone }
                    ) if zone == "main_tavern"
                ))
                .count(),
            10
        );
        assert!(blueprint.props.iter().any(|prop| matches!(
            (&prop.visual, &prop.cutaway),
            (
                PropVisual::Sprite {
                    asset_id: Some(asset_id),
                    ..
                },
                _
            ) if asset_id == "prop.bar_counter"
        )));
        assert!(blueprint.props.iter().any(|prop| matches!(
            (&prop.visual, &prop.cutaway),
            (
                PropVisual::Sprite {
                    layer: UniverseLayer::Roof,
                    ..
                },
                CutawayRule::HideWhenPlayerInside { zone }
            ) if zone == "main_tavern"
        )));

        let world_map = blueprint.build_world_map();
        assert!(world_map.is_blocked(5, 4));
        assert!(world_map.is_blocked(8, 6));
        assert!(world_map.is_blocked(9, 7));
    }

    #[test]
    fn world_document_serializes_after_prefab_edit() {
        let mut document = load_world_document(DEMO_TAVERN_PATH).expect("demo tavern should load");
        document
            .prefab_instances
            .push(crate::world::prefabs::PrefabInstanceFile::new(
                "table_001".to_string(),
                "prop.table_round".to_string(),
                Vec2::new(10.0, 9.0),
            ));

        let serialized =
            ron::ser::to_string(&document).expect("edited world document should serialize");
        let reparsed: WorldDocument =
            ron::de::from_str(&serialized).expect("serialized world document should reparse");

        assert_eq!(reparsed.prefab_instances.len(), 1);
        assert_eq!(reparsed.prefab_instances[0].id, "table_001");
    }

    #[test]
    fn empty_scene_starts_without_ground_tiles() {
        let asset_registry =
            crate::rendering::load_asset_registry(crate::rendering::ASSET_REGISTRY_PATH)
                .expect("asset registry should load");
        let prefab_catalog = crate::world::prefabs::load_prefab_catalog(
            crate::world::prefabs::PREFAB_CATALOG_PATH,
            &asset_registry,
        )
        .expect("prefab catalog should load");
        let blueprint = WorldDocument::empty_scene(12, 8, 1)
            .into_blueprint(&asset_registry, &prefab_catalog)
            .expect("empty scene should still be a valid world");

        assert_eq!(blueprint.width, 12);
        assert_eq!(blueprint.height, 8);
        assert!(blueprint.tiles.is_empty());
        assert!(blueprint.props.is_empty());
    }

    #[test]
    fn multi_level_world_builds_level_specific_tiles_and_collision() {
        let asset_registry =
            crate::rendering::load_asset_registry(crate::rendering::ASSET_REGISTRY_PATH)
                .expect("asset registry should load");
        let prefab_catalog = crate::world::prefabs::load_prefab_catalog(
            crate::world::prefabs::PREFAB_CATALOG_PATH,
            &asset_registry,
        )
        .expect("prefab catalog should load");
        let mut document = WorldDocument::empty_demo();
        document.levels = 2;
        document.tile_regions.push(TileRegionFile {
            kind: TileKindFile::TavernFloor,
            z: 1,
            from: vec![2, 2],
            to: vec![3, 3],
        });
        document
            .collision_cells
            .push(CollisionCellFile::at(IVec2::new(4, 4), 1, true));
        document
            .prefab_instances
            .push(crate::world::prefabs::PrefabInstanceFile::at(
                "table_upper_001".to_string(),
                "prop.table_round".to_string(),
                Vec2::new(2.0, 2.0),
                1,
            ));
        document.edge_walls.push(EdgeWallFile {
            grid: vec![1, 1],
            z: 1,
            direction: EdgeDirectionFile::S,
            style: PropStyleRefFile::Inline(PropStyleFile {
                size: vec![64.0, 48.0],
                layer: PropLayerFile::Wall,
                asset_id: String::new(),
                asset: String::new(),
                color: vec![0.2, 0.2, 0.2],
                anchor: RenderAnchorFile::WallBaseline,
                y_offset: None,
                depth_offset: 0.0,
                shadow: false,
                footprint: FootprintFile::Nearest,
                cutaway: CutawayFile::None,
            }),
        });
        document.vertical_links.push(VerticalLinkFile {
            id: "stairs.test".to_string(),
            kind: VerticalLinkKindFile::Stairs,
            lower: NavPointFile::Detailed {
                grid: vec![1.0, 1.0],
                z: 0,
            },
            upper: NavPointFile::Detailed {
                grid: vec![1.0, 1.0],
                z: 1,
            },
            bidirectional: true,
        });
        document.seats.push(NavPointFile::Detailed {
            grid: vec![3.0, 3.0],
            z: 1,
        });

        let blueprint = document
            .into_blueprint(&asset_registry, &prefab_catalog)
            .expect("multi-level document should build");
        let world_map = blueprint.build_world_map();

        assert_eq!(blueprint.levels, 2);
        assert_eq!(blueprint.collision_cells.len(), 1);
        assert_eq!(blueprint.edge_walls.len(), 1);
        assert_eq!(blueprint.vertical_links.len(), 1);
        assert_eq!(blueprint.seats[0].z, 1);
        assert!(
            blueprint
                .tiles
                .iter()
                .any(|tile| tile.z == 1 && tile.grid == IVec2::new(2, 2))
        );
        assert!(!world_map.is_blocked_at(2, 2, 0));
        assert!(world_map.is_blocked_at(2, 2, 1));
        assert!(world_map.is_blocked_at(4, 4, 1));
        assert!(!world_map.is_edge_blocked_between_at(IVec2::new(1, 1), IVec2::new(2, 1), 0));
        assert!(world_map.is_edge_blocked_between_at(IVec2::new(1, 1), IVec2::new(2, 1), 1));
    }
}
