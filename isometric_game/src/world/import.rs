use std::collections::HashMap;
use std::fs;
use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

use crate::gameplay::StationKind;
use crate::rendering::RenderAnchor;
use crate::universe::UniverseLayer;
use crate::world::blueprint::{
    CutawayRule, Footprint, PropPlacement, PropVisual, ShadowSpec, StationPlacement, TileKind,
    TilePlacement, WorldBlueprint, WorldZone,
};

pub const DEMO_TAVERN_PATH: &str = "data/worlds/demo_tavern.ron";

pub fn load_world_blueprint(path: &str) -> Result<WorldBlueprint, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let file: WorldFile = ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    file.into_blueprint()
}

#[derive(Deserialize)]
struct WorldFile {
    width: i32,
    height: i32,
    #[serde(default)]
    prop_styles: Vec<PropStyleDefinitionFile>,
    tile_regions: Vec<TileRegionFile>,
    #[serde(default)]
    zones: Vec<ZoneFile>,
    #[serde(default)]
    props: Vec<PropFile>,
    #[serde(default)]
    prop_runs: Vec<PropRunFile>,
    #[serde(default)]
    stations: Vec<StationFile>,
    #[serde(default)]
    seats: Vec<Vec<f32>>,
    entrance: Vec<f32>,
}

impl WorldFile {
    fn into_blueprint(self) -> Result<WorldBlueprint, String> {
        if self.width <= 0 || self.height <= 0 {
            return Err(format!(
                "world dimensions must be positive, got {}x{}",
                self.width, self.height
            ));
        }

        let styles = StyleLibrary::from_definitions(self.prop_styles)?;
        let mut blueprint = WorldBlueprint::new(self.width, self.height);
        add_tiles(&mut blueprint, self.width, self.height, &self.tile_regions)?;

        for zone in self.zones {
            blueprint.add_zone(zone.into_zone()?);
        }
        for prop in self.props {
            blueprint.add_prop(prop.into_prop(&styles)?);
        }
        for run in self.prop_runs {
            for prop in run.into_props(&styles)? {
                blueprint.add_prop(prop);
            }
        }
        for station in self.stations {
            blueprint.add_station(station.into_station(&styles)?);
        }
        for seat in self.seats {
            blueprint.add_seat(vec2(&seat, "seat marker")?);
        }

        blueprint.set_entrance(vec2(&self.entrance, "entrance marker")?);

        Ok(blueprint)
    }
}

#[derive(Deserialize)]
struct ZoneFile {
    name: String,
    min: Vec<f32>,
    max: Vec<f32>,
}

impl ZoneFile {
    fn into_zone(self) -> Result<WorldZone, String> {
        let min = vec2(&self.min, "zone min")?;
        let max = vec2(&self.max, "zone max")?;
        if min.x > max.x || min.y > max.y {
            return Err(format!(
                "zone '{}' has invalid bounds {:?} -> {:?}",
                self.name, min, max
            ));
        }

        Ok(WorldZone::new(self.name, min, max))
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
    regions: &[TileRegionFile],
) -> Result<(), String> {
    let mut tiles = vec![TileKind::Ground; (width * height) as usize];

    for region in regions {
        let from = ivec2(&region.from, "tile region from")?;
        let to = ivec2(&region.to, "tile region to")?;
        if from.x > to.x || from.y > to.y {
            return Err(format!(
                "invalid tile region {:?} -> {:?}",
                region.from, region.to
            ));
        }

        for y in from.y..=to.y {
            for x in from.x..=to.x {
                if x < 0 || y < 0 || x >= width || y >= height {
                    continue;
                }
                tiles[(y * width + x) as usize] = region.kind.into_tile_kind();
            }
        }
    }

    for y in 0..height {
        for x in 0..width {
            blueprint.add_tile(TilePlacement::new(
                IVec2::new(x, y),
                tiles[(y * width + x) as usize],
            ));
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct TileRegionFile {
    kind: TileKindFile,
    from: Vec<i32>,
    to: Vec<i32>,
}

#[derive(Clone, Copy, Deserialize)]
enum TileKindFile {
    Ground,
    TavernFloor,
    Porch,
}

impl TileKindFile {
    fn into_tile_kind(self) -> TileKind {
        match self {
            TileKindFile::Ground => TileKind::Ground,
            TileKindFile::TavernFloor => TileKind::TavernFloor,
            TileKindFile::Porch => TileKind::Porch,
        }
    }
}

#[derive(Deserialize)]
struct PropFile {
    grid: Vec<f32>,
    style: PropStyleRefFile,
}

impl PropFile {
    fn into_prop(self, styles: &StyleLibrary) -> Result<PropPlacement, String> {
        let grid = vec2(&self.grid, "prop grid")?;
        styles.resolve(self.style, "prop")?.into_prop(grid)
    }
}

#[derive(Deserialize)]
struct PropRunFile {
    start: Vec<f32>,
    count: usize,
    step: Vec<f32>,
    style: PropStyleRefFile,
}

impl PropRunFile {
    fn into_props(self, styles: &StyleLibrary) -> Result<Vec<PropPlacement>, String> {
        let mut props = Vec::with_capacity(self.count);
        let start = vec2(&self.start, "prop run start")?;
        let step = vec2(&self.step, "prop run step")?;
        let style = styles.resolve(self.style, "prop run")?;

        for index in 0..self.count {
            props.push(style.clone().into_prop(start + step * index as f32)?);
        }

        Ok(props)
    }
}

#[derive(Deserialize)]
struct PropStyleDefinitionFile {
    name: String,
    style: PropStyleFile,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum PropStyleRefFile {
    Named(String),
    Inline(PropStyleFile),
}

#[derive(Clone, Deserialize)]
struct PropStyleFile {
    size: Vec<f32>,
    layer: PropLayerFile,
    #[serde(default)]
    asset: String,
    #[serde(default)]
    color: Vec<f32>,
    #[serde(default)]
    anchor: RenderAnchorFile,
    #[serde(default)]
    y_offset: Option<f32>,
    #[serde(default)]
    depth_offset: f32,
    #[serde(default)]
    shadow: bool,
    #[serde(default)]
    footprint: FootprintFile,
    #[serde(default)]
    cutaway: CutawayFile,
}

impl PropStyleFile {
    fn into_prop(self, grid: Vec2) -> Result<PropPlacement, String> {
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

        let visual = if !self.asset.is_empty() {
            PropVisual::Sprite {
                asset_path: self.asset,
                size,
                anchor,
                layer,
                depth_offset: self.depth_offset,
                shadow,
            }
        } else {
            if self.color.is_empty() {
                return Err(format!("prop at {grid:?} needs either asset or color"));
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
            grid,
            visual,
            footprint: self.footprint.into_footprint()?,
            cutaway: self.cutaway.into_cutaway_rule(),
        })
    }
}

#[derive(Clone, Copy, Deserialize, Default)]
enum RenderAnchorFile {
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

#[derive(Clone, Copy, Deserialize)]
enum PropLayerFile {
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

#[derive(Clone, Deserialize, Default)]
enum CutawayFile {
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

#[derive(Clone, Deserialize, Default)]
enum FootprintFile {
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

#[derive(Deserialize)]
struct StationFile {
    kind: StationKindFile,
    grid: Vec<f32>,
    style: PropStyleRefFile,
}

impl StationFile {
    fn into_station(self, styles: &StyleLibrary) -> Result<StationPlacement, String> {
        let grid = vec2(&self.grid, "station grid")?;
        Ok(StationPlacement::new(
            self.kind.into_station_kind(),
            styles.resolve(self.style, "station")?.into_prop(grid)?,
        ))
    }
}

#[derive(Deserialize)]
enum StationKindFile {
    Pantry,
    WaterStill,
    Stove,
    Radio,
}

impl StationKindFile {
    fn into_station_kind(self) -> StationKind {
        match self {
            StationKindFile::Pantry => StationKind::Pantry,
            StationKindFile::WaterStill => StationKind::WaterStill,
            StationKindFile::Stove => StationKind::Stove,
            StationKindFile::Radio => StationKind::Radio,
        }
    }
}

fn ivec2(value: &[i32], label: &str) -> Result<IVec2, String> {
    if value.len() != 2 {
        return Err(format!("{label} must have exactly 2 values, got {value:?}"));
    }

    Ok(IVec2::new(value[0], value[1]))
}

fn vec2(value: &[f32], label: &str) -> Result<Vec2, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_tavern_file_loads_as_blueprint() {
        let blueprint = load_world_blueprint(DEMO_TAVERN_PATH).expect("demo tavern should load");

        assert_eq!(blueprint.width, 26);
        assert_eq!(blueprint.height, 18);
        assert_eq!(blueprint.tiles.len(), 26 * 18);
        assert_eq!(blueprint.zones.len(), 1);
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
}
