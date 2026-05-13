mod blueprint;
mod camera;
mod cutaway;
mod editor;
mod import;
mod layout;
mod prefabs;
mod runtime;
mod spawn;

use bevy::prelude::*;

use crate::GameSet;

pub use blueprint::{NavPoint, TavernMap, VerticalLink, WorldBlueprint};
pub use editor::SceneEditorState;
pub use import::WorldDocument;
pub use prefabs::PrefabCatalog;
pub use runtime::{UNTITLED_SCENE_PATH, WorldRuntimeConfig, WorldRuntimeStatus};

use camera::{camera_follow_player, camera_zoom, setup_camera};
use cutaway::update_cutaways;
use editor::{
    SceneSelectorState, draw_scene_editor, scene_editor_button_input, scene_editor_input,
    scene_editor_mouse_input, scene_selector_button_input, setup_scene_editor_ui,
    setup_scene_selector_ui, sync_scene_selector_ui, update_scene_editor_ui_visibility,
    update_scene_selector_ui_visibility,
};
use runtime::{
    WorldReloadRequested, fallback_world_bundle, handle_world_reload_requests, load_world_bundle,
};
use spawn::spawn_world;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let config = app
            .world()
            .get_resource::<WorldRuntimeConfig>()
            .cloned()
            .unwrap_or_default();
        let editor_enabled = config.editor_enabled;
        let scene_selector_enabled = config.scene_selector_enabled;
        let loaded = load_world_bundle(&config)
            .unwrap_or_else(|error| fallback_world_bundle(&config, error));

        app.insert_resource(config)
            .insert_resource(loaded.document)
            .insert_resource(loaded.blueprint)
            .insert_resource(loaded.asset_registry)
            .insert_resource(loaded.prefab_catalog)
            .insert_resource(loaded.world_map)
            .insert_resource(SceneEditorState::new(editor_enabled))
            .insert_resource(SceneSelectorState::new(scene_selector_enabled))
            .init_resource::<WorldRuntimeStatus>()
            .add_event::<WorldReloadRequested>()
            .add_systems(
                Startup,
                (
                    setup_camera,
                    spawn_world,
                    setup_scene_editor_ui,
                    setup_scene_selector_ui,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    scene_editor_input.in_set(GameSet::Input),
                    scene_selector_button_input.in_set(GameSet::Input),
                    scene_editor_button_input.in_set(GameSet::Input),
                    scene_editor_mouse_input.in_set(GameSet::Input),
                    handle_world_reload_requests.in_set(GameSet::Gameplay),
                    sync_scene_selector_ui.in_set(GameSet::Ui),
                    update_scene_editor_ui_visibility.in_set(GameSet::Ui),
                    update_scene_selector_ui_visibility.in_set(GameSet::Ui),
                    draw_scene_editor.in_set(GameSet::Debug),
                ),
            )
            .add_systems(Update, update_cutaways.in_set(GameSet::RenderSync))
            .add_systems(
                Update,
                (camera_follow_player, camera_zoom)
                    .chain()
                    .in_set(GameSet::Camera),
            );
    }
}
