use std::collections::BTreeMap;
use std::fs;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rendering::AssetRegistry;
use crate::world::blueprint::{StationPlacement, WorldBlueprint};
use crate::world::import::{PropStyleFile, StationKindFile, project_path, vec2};

pub const PREFAB_CATALOG_PATH: &str = "data/prefabs/world_objects.ron";

#[derive(Resource, Clone, Default)]
pub struct PrefabCatalog {
    prefabs: BTreeMap<String, PrefabDefinition>,
    palette: Vec<String>,
}

impl PrefabCatalog {
    pub(crate) fn get(&self, id: &str) -> Option<&PrefabDefinition> {
        self.prefabs.get(id)
    }

    pub fn palette_ids(&self) -> &[String] {
        &self.palette
    }

    pub fn label_for<'a>(&'a self, id: &'a str) -> &'a str {
        self.get(id)
            .map(|prefab| prefab.label.as_str())
            .unwrap_or(id)
    }

    pub fn add_instance_to_blueprint(
        &self,
        instance: PrefabInstanceFile,
        asset_registry: &AssetRegistry,
        blueprint: &mut WorldBlueprint,
    ) -> Result<(), String> {
        let prefab = self.get(&instance.prefab).ok_or_else(|| {
            format!(
                "prefab instance '{}' references unknown prefab '{}'",
                instance.id, instance.prefab
            )
        })?;

        let grid = vec2(&instance.grid, "prefab instance grid")?;
        let prop = prefab
            .style
            .clone()
            .into_prop(grid, instance.z, asset_registry)?
            .with_source(instance.id.clone(), instance.prefab.clone());

        if let Some(station_kind) = prefab.station {
            blueprint.add_station(StationPlacement::new(
                station_kind.into_station_kind(),
                prop,
            ));
        } else {
            blueprint.add_prop(prop);
        }

        Ok(())
    }

    fn from_file(file: PrefabCatalogFile) -> Result<Self, String> {
        let mut prefabs = BTreeMap::new();
        let mut palette = Vec::new();

        for definition in file.prefabs {
            if definition.id.trim().is_empty() {
                return Err("prefab id cannot be empty".to_string());
            }
            if definition.label.trim().is_empty() {
                return Err(format!("prefab '{}' must have a label", definition.id));
            }

            let id = definition.id.clone();
            if definition.show_in_palette {
                palette.push(id.clone());
            }
            if prefabs.insert(id.clone(), definition).is_some() {
                return Err(format!("duplicate prefab id '{id}'"));
            }
        }

        if prefabs.is_empty() {
            return Err("prefab catalog must contain at least one prefab".to_string());
        }
        if palette.is_empty() {
            palette.extend(prefabs.keys().cloned());
        }

        Ok(Self { prefabs, palette })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PrefabDefinition {
    pub id: String,
    pub label: String,
    pub style: PropStyleFile,
    #[serde(default)]
    pub station: Option<StationKindFile>,
    #[serde(default = "default_show_in_palette")]
    pub show_in_palette: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PrefabInstanceFile {
    pub id: String,
    pub prefab: String,
    pub grid: Vec<f32>,
    #[serde(default = "default_z")]
    pub z: i32,
}

impl PrefabInstanceFile {
    #[cfg(test)]
    pub fn new(id: String, prefab: String, grid: Vec2) -> Self {
        Self::at(id, prefab, grid, 0)
    }

    pub fn at(id: String, prefab: String, grid: Vec2, z: i32) -> Self {
        Self {
            id,
            prefab,
            grid: vec![grid.x, grid.y],
            z,
        }
    }

    pub fn grid_vec(&self) -> Result<Vec2, String> {
        vec2(&self.grid, "prefab instance grid")
    }

    pub fn set_grid(&mut self, grid: Vec2) {
        self.grid = vec![grid.x, grid.y];
    }

    pub fn set_level(&mut self, z: i32) {
        self.z = z;
    }
}

#[derive(Deserialize)]
struct PrefabCatalogFile {
    #[serde(default)]
    prefabs: Vec<PrefabDefinition>,
}

pub fn load_prefab_catalog(
    path: &str,
    asset_registry: &AssetRegistry,
) -> Result<PrefabCatalog, String> {
    let path = project_path(path);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let file: PrefabCatalogFile = ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    let catalog = PrefabCatalog::from_file(file)?;
    validate_prefab_assets(&catalog, asset_registry)?;
    Ok(catalog)
}

fn validate_prefab_assets(
    catalog: &PrefabCatalog,
    asset_registry: &AssetRegistry,
) -> Result<(), String> {
    for prefab in catalog.prefabs.values() {
        if !prefab.style.asset_id.is_empty() {
            asset_registry
                .sprite_path(&prefab.style.asset_id)
                .map_err(|error| format!("prefab '{}': {error}", prefab.id))?;
        }
    }

    Ok(())
}

fn default_show_in_palette() -> bool {
    true
}

fn default_z() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{ASSET_REGISTRY_PATH, load_asset_registry};

    #[test]
    fn prefab_catalog_loads_and_validates_assets() {
        let asset_registry =
            load_asset_registry(ASSET_REGISTRY_PATH).expect("asset registry should load");
        let catalog = load_prefab_catalog(PREFAB_CATALOG_PATH, &asset_registry)
            .expect("prefab catalog should load");

        assert!(catalog.get("prop.table_round").is_some());
        assert!(catalog.palette_ids().contains(&"prop.stove".to_string()));
    }

    #[test]
    fn prefab_instances_build_props_and_stations() {
        let asset_registry =
            load_asset_registry(ASSET_REGISTRY_PATH).expect("asset registry should load");
        let catalog = load_prefab_catalog(PREFAB_CATALOG_PATH, &asset_registry)
            .expect("prefab catalog should load");
        let mut document = crate::world::import::WorldDocument::empty_demo();
        document.prefab_instances.push(PrefabInstanceFile::new(
            "table_001".to_string(),
            "prop.table_round".to_string(),
            Vec2::new(8.0, 8.0),
        ));
        document.prefab_instances.push(PrefabInstanceFile::new(
            "stove_001".to_string(),
            "prop.stove".to_string(),
            Vec2::new(9.0, 8.0),
        ));

        let blueprint = document
            .into_blueprint(&asset_registry, &catalog)
            .expect("prefab instances should build");

        assert_eq!(blueprint.props.len(), 1);
        assert_eq!(blueprint.stations.len(), 1);
        assert_eq!(blueprint.props[0].instance_id.as_deref(), Some("table_001"));
        assert_eq!(
            blueprint.stations[0].prop.instance_id.as_deref(),
            Some("stove_001")
        );
    }
}
