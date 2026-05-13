use bevy::prelude::*;

use crate::rendering::{ASSET_REGISTRY_PATH, AssetRegistry, load_asset_registry};
use crate::universe::WorldMap;
use crate::world::blueprint::{WorldBlueprint, WorldSceneEntity};
use crate::world::import::{
    DEMO_TAVERN_PATH, WorldDocument, load_world_document, save_world_document,
};
use crate::world::layout::demo_tavern_blueprint;
use crate::world::prefabs::{PREFAB_CATALOG_PATH, PrefabCatalog, load_prefab_catalog};
use crate::world::spawn::spawn_world_entities;

pub const UNTITLED_SCENE_PATH: &str = "data/worlds/untitled_scene.ron";

#[derive(Resource, Clone)]
pub struct WorldRuntimeConfig {
    pub world_path: String,
    pub asset_registry_path: String,
    pub prefab_catalog_path: String,
    pub editor_enabled: bool,
    pub scene_selector_enabled: bool,
    pub start_empty_scene: bool,
}

impl Default for WorldRuntimeConfig {
    fn default() -> Self {
        Self {
            world_path: DEMO_TAVERN_PATH.to_string(),
            asset_registry_path: ASSET_REGISTRY_PATH.to_string(),
            prefab_catalog_path: PREFAB_CATALOG_PATH.to_string(),
            editor_enabled: false,
            scene_selector_enabled: false,
            start_empty_scene: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct WorldRuntimeStatus {
    pub last_message: String,
}

#[derive(Event)]
pub struct WorldReloadRequested {
    pub source: WorldReloadSource,
    pub reason: String,
}

impl WorldReloadRequested {
    pub fn disk(reason: impl Into<String>) -> Self {
        Self {
            source: WorldReloadSource::Disk,
            reason: reason.into(),
        }
    }

    pub fn current_document(reason: impl Into<String>) -> Self {
        Self {
            source: WorldReloadSource::CurrentDocument,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum WorldReloadSource {
    Disk,
    CurrentDocument,
}

pub struct LoadedWorld {
    pub asset_registry: AssetRegistry,
    pub prefab_catalog: PrefabCatalog,
    pub document: WorldDocument,
    pub blueprint: WorldBlueprint,
    pub world_map: WorldMap,
}

pub fn load_world_bundle(config: &WorldRuntimeConfig) -> Result<LoadedWorld, String> {
    let asset_registry = load_asset_registry(&config.asset_registry_path)?;
    let prefab_catalog = load_prefab_catalog(&config.prefab_catalog_path, &asset_registry)?;
    if config.start_empty_scene {
        return Ok(empty_world_bundle(
            asset_registry,
            prefab_catalog,
            26,
            18,
            1,
        ));
    }

    let document = load_world_document(&config.world_path)?;
    let blueprint = document
        .clone()
        .into_blueprint(&asset_registry, &prefab_catalog)?;
    let world_map = blueprint.build_world_map();

    Ok(LoadedWorld {
        asset_registry,
        prefab_catalog,
        document,
        blueprint,
        world_map,
    })
}

pub fn empty_world_bundle(
    asset_registry: AssetRegistry,
    prefab_catalog: PrefabCatalog,
    width: i32,
    height: i32,
    levels: i32,
) -> LoadedWorld {
    let document = WorldDocument::empty_scene(width, height, levels);
    let blueprint = document
        .clone()
        .into_blueprint(&asset_registry, &prefab_catalog)
        .expect("empty editor scene should build");
    let world_map = blueprint.build_world_map();

    LoadedWorld {
        asset_registry,
        prefab_catalog,
        document,
        blueprint,
        world_map,
    }
}

pub fn fallback_world_bundle(config: &WorldRuntimeConfig, error: String) -> LoadedWorld {
    eprintln!("Falling back to built-in demo tavern blueprint: {error}");

    let asset_registry = load_asset_registry(&config.asset_registry_path).unwrap_or_else(|error| {
        eprintln!("Asset registry unavailable; legacy asset paths only: {error}");
        AssetRegistry::default()
    });
    let prefab_catalog = load_prefab_catalog(&config.prefab_catalog_path, &asset_registry)
        .unwrap_or_else(|error| {
            eprintln!("Prefab catalog unavailable; editor palette disabled: {error}");
            PrefabCatalog::default()
        });
    let document = WorldDocument::empty_demo();
    let blueprint = demo_tavern_blueprint();
    let world_map = blueprint.build_world_map();

    LoadedWorld {
        asset_registry,
        prefab_catalog,
        document,
        blueprint,
        world_map,
    }
}

pub fn handle_world_reload_requests(
    mut commands: Commands,
    mut requests: EventReader<WorldReloadRequested>,
    config: Res<WorldRuntimeConfig>,
    asset_server: Res<AssetServer>,
    mut asset_registry: ResMut<AssetRegistry>,
    mut prefab_catalog: ResMut<PrefabCatalog>,
    mut document: ResMut<WorldDocument>,
    mut blueprint: ResMut<WorldBlueprint>,
    mut world_map: ResMut<WorldMap>,
    mut status: ResMut<WorldRuntimeStatus>,
    scene_entities: Query<Entity, With<WorldSceneEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(request) = requests.read().last() else {
        return;
    };

    let result = match request.source {
        WorldReloadSource::Disk => load_world_bundle(&config),
        WorldReloadSource::CurrentDocument => {
            let new_blueprint = document
                .clone()
                .into_blueprint(&asset_registry, &prefab_catalog);
            new_blueprint.map(|blueprint| LoadedWorld {
                asset_registry: asset_registry.clone(),
                prefab_catalog: prefab_catalog.clone(),
                document: document.clone(),
                world_map: blueprint.build_world_map(),
                blueprint,
            })
        }
    };

    let loaded = match result {
        Ok(loaded) => loaded,
        Err(error) => {
            status.last_message = format!("Reload failed: {error}");
            eprintln!("{}", status.last_message);
            return;
        }
    };

    for entity in &scene_entities {
        commands.entity(entity).despawn_recursive();
    }

    *asset_registry = loaded.asset_registry;
    *prefab_catalog = loaded.prefab_catalog;
    *document = loaded.document;
    *world_map = loaded.world_map;
    *blueprint = loaded.blueprint;

    spawn_world_entities(
        &mut commands,
        &asset_server,
        &asset_registry,
        &blueprint,
        &mut meshes,
        &mut materials,
    );

    status.last_message = format!("Reloaded world/assets: {}", request.reason);
    println!("{}", status.last_message);
}

pub fn save_current_world_document(
    config: &WorldRuntimeConfig,
    document: &WorldDocument,
) -> Result<(), String> {
    save_world_document(&config.world_path, document)
}
