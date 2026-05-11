use std::fs;
use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

use crate::gameplay::StationKind;
use crate::universe::UniverseLayer;
use crate::world::blueprint::{
    Footprint, PropPlacement, PropVisual, ShadowSpec, StationPlacement, TileKind, TilePlacement,
    WorldBlueprint,
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
    tile_regions: Vec<TileRegionFile>,
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

        let mut blueprint = WorldBlueprint::new(self.width, self.height);
        add_tiles(&mut blueprint, self.width, self.height, &self.tile_regions)?;

        for prop in self.props {
            blueprint.add_prop(prop.into_prop()?);
        }
        for run in self.prop_runs {
            for prop in run.into_props()? {
                blueprint.add_prop(prop);
            }
        }
        for station in self.stations {
            blueprint.add_station(station.into_station()?);
        }
        for seat in self.seats {
            blueprint.add_seat(vec2(&seat, "seat marker")?);
        }

        blueprint.set_entrance(vec2(&self.entrance, "entrance marker")?);

        Ok(blueprint)
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
    style: PropStyleFile,
}

impl PropFile {
    fn into_prop(self) -> Result<PropPlacement, String> {
        self.style.into_prop(vec2(&self.grid, "prop grid")?)
    }
}

#[derive(Deserialize)]
struct PropRunFile {
    start: Vec<f32>,
    count: usize,
    step: Vec<f32>,
    style: PropStyleFile,
}

impl PropRunFile {
    fn into_props(self) -> Result<Vec<PropPlacement>, String> {
        let mut props = Vec::with_capacity(self.count);
        let start = vec2(&self.start, "prop run start")?;
        let step = vec2(&self.step, "prop run step")?;

        for index in 0..self.count {
            props.push(self.style.clone().into_prop(start + step * index as f32)?);
        }

        Ok(props)
    }
}

#[derive(Clone, Deserialize)]
struct PropStyleFile {
    size: Vec<f32>,
    color: Vec<f32>,
    layer: PropLayerFile,
    #[serde(default)]
    y_offset: Option<f32>,
    #[serde(default)]
    depth_offset: f32,
    #[serde(default)]
    shadow: bool,
    #[serde(default)]
    footprint: FootprintFile,
}

impl PropStyleFile {
    fn into_prop(self, grid: Vec2) -> Result<PropPlacement, String> {
        let size = vec2(&self.size, "prop size")?;
        if size.x <= 0.0 || size.y <= 0.0 {
            return Err(format!("prop at {grid:?} has invalid size {size:?}"));
        }

        let layer = self.layer.into_universe_layer();
        let y_offset = self.y_offset.unwrap_or_else(|| match layer {
            UniverseLayer::Wall => size.y * 0.5,
            _ => size.y * 0.45,
        });
        let shadow = self.shadow.then_some(ShadowSpec {
            size: Vec2::new(size.x * 0.95, 13.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.34),
            y_offset: 3.0,
            depth_offset: self.depth_offset - 1.0,
        });

        Ok(PropPlacement {
            grid,
            visual: PropVisual::MeshRect {
                size,
                color: color(&self.color, "prop color")?,
                y_offset,
                layer,
                depth_offset: self.depth_offset,
                shadow,
            },
            footprint: self.footprint.into_footprint()?,
        })
    }
}

#[derive(Clone, Copy, Deserialize)]
enum PropLayerFile {
    Prop,
    Wall,
}

impl PropLayerFile {
    fn into_universe_layer(self) -> UniverseLayer {
        match self {
            PropLayerFile::Prop => UniverseLayer::Prop,
            PropLayerFile::Wall => UniverseLayer::Wall,
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
    style: PropStyleFile,
}

impl StationFile {
    fn into_station(self) -> Result<StationPlacement, String> {
        Ok(StationPlacement::new(
            self.kind.into_station_kind(),
            self.style.into_prop(vec2(&self.grid, "station grid")?)?,
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

fn color(value: &[f32], label: &str) -> Result<Color, String> {
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
        assert_eq!(blueprint.stations.len(), 4);
        assert_eq!(blueprint.seats.len(), 6);
        assert!(!blueprint.props.is_empty());

        let world_map = blueprint.build_world_map();
        assert!(world_map.is_blocked(5, 4));
        assert!(world_map.is_blocked(8, 7));
    }
}
