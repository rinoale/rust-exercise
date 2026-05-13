use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};

use bevy::prelude::*;
use serde::Deserialize;

pub const ASSET_REGISTRY_PATH: &str = "data/assets/registry.ron";

#[derive(Resource, Clone, Default)]
pub struct AssetRegistry {
    sprites: BTreeMap<String, SpriteAsset>,
}

impl AssetRegistry {
    pub fn sprite(&self, id: &str) -> Option<&SpriteAsset> {
        self.sprites.get(id)
    }

    pub fn sprite_path(&self, id: &str) -> Result<&str, String> {
        self.sprite(id)
            .map(|sprite| sprite.path.as_str())
            .ok_or_else(|| format!("unknown sprite asset id '{id}'"))
    }

    pub fn sprite_path_or_fallback<'a>(&'a self, id: &str, fallback: &'a str) -> &'a str {
        self.sprite(id)
            .map(|sprite| sprite.path.as_str())
            .unwrap_or(fallback)
    }

    pub fn sprites_by_kind(
        &self,
        kind: SpriteAssetKind,
    ) -> impl Iterator<Item = &SpriteAsset> + '_ {
        self.sprites
            .values()
            .filter(move |sprite| sprite.kind == kind)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    fn validate_baseline(&self) -> Result<(), String> {
        if self.is_empty() {
            return Err("asset registry must contain at least one sprite".to_string());
        }

        for (id, sprite) in &self.sprites {
            if sprite.id != *id {
                return Err(format!(
                    "sprite asset map key '{}' does not match stored id '{}'",
                    id, sprite.id
                ));
            }
        }

        for kind in [
            SpriteAssetKind::Tile,
            SpriteAssetKind::Wall,
            SpriteAssetKind::Roof,
            SpriteAssetKind::Prop,
        ] {
            if self.sprites_by_kind(kind).next().is_none() {
                return Err(format!(
                    "asset registry needs at least one {kind:?} sprite for world authoring"
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct SpriteAsset {
    pub id: String,
    pub kind: SpriteAssetKind,
    pub path: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum SpriteAssetKind {
    Tile,
    Wall,
    Roof,
    Prop,
    Decal,
    Character,
    Item,
    Ui,
}

pub fn load_asset_registry(path: &str) -> Result<AssetRegistry, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let file: AssetRegistryFile = ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    file.into_registry()
}

#[derive(Deserialize)]
struct AssetRegistryFile {
    #[serde(default)]
    sprites: Vec<SpriteAssetFile>,
}

impl AssetRegistryFile {
    fn into_registry(self) -> Result<AssetRegistry, String> {
        let mut sprites = BTreeMap::new();
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

        for sprite in self.sprites {
            sprite.validate(&asset_root)?;

            let asset = SpriteAsset {
                id: sprite.id.clone(),
                kind: sprite.kind,
                path: sprite.path,
            };

            if sprites.insert(sprite.id.clone(), asset).is_some() {
                return Err(format!("duplicate sprite asset id '{}'", sprite.id));
            }
        }

        let registry = AssetRegistry { sprites };
        registry.validate_baseline()?;
        Ok(registry)
    }
}

#[derive(Deserialize)]
struct SpriteAssetFile {
    id: String,
    kind: SpriteAssetKind,
    path: String,
}

impl SpriteAssetFile {
    fn validate(&self, asset_root: &Path) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("sprite asset id cannot be empty".to_string());
        }
        if self.path.trim().is_empty() {
            return Err(format!("sprite asset '{}' has an empty path", self.id));
        }

        let path = Path::new(&self.path);
        if path.is_absolute() {
            return Err(format!(
                "sprite asset '{}' path must be relative to assets/: {}",
                self.id, self.path
            ));
        }
        if path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(format!(
                "sprite asset '{}' path cannot leave assets/: {}",
                self.id, self.path
            ));
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("png") {
            return Err(format!(
                "sprite asset '{}' must point at a png file: {}",
                self.id, self.path
            ));
        }

        let full_path = asset_root.join(path);
        if !full_path.is_file() {
            return Err(format!(
                "sprite asset '{}' points at missing file {}",
                self.id,
                full_path.display()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_registry_loads_known_assets() {
        let registry =
            load_asset_registry(ASSET_REGISTRY_PATH).expect("runtime asset registry should load");

        assert!(registry.len() >= 1);
        assert_eq!(
            registry.sprite_path("prop.bar_counter").unwrap(),
            "props/bar_counter.png"
        );
        assert_eq!(
            registry.sprite_path("tile.tavern.floor_worn_a").unwrap(),
            "tiles/tavern_floor_worn_a.png"
        );
        assert_eq!(
            registry
                .sprites_by_kind(SpriteAssetKind::Wall)
                .filter(|sprite| sprite.id.starts_with("wall."))
                .count(),
            7
        );
    }
}
