use std::fs;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::config::{TILE_HEIGHT, TILE_WIDTH};
use crate::rendering::AssetRegistry;
use crate::universe::{GridPosition, grid_to_screen_at, screen_to_grid_at};
use crate::world::blueprint::WorldObjectEntity;
use crate::world::import::{
    CollisionCellFile, EdgeDirectionFile, EdgeWallFile, NavPointFile, PropLayerFile,
    PropStyleRefFile, TileKindFile, TileRegionFile, VerticalLinkFile, VerticalLinkKindFile,
    WorldDocument, ZoneFile, project_path,
};
use crate::world::prefabs::{PrefabCatalog, PrefabInstanceFile};
use crate::world::runtime::{
    UNTITLED_SCENE_PATH, WorldReloadRequested, WorldRuntimeConfig, WorldRuntimeStatus,
    save_current_world_document,
};

use super::camera::MainCamera;

const EDITOR_TOP_BAR_HEIGHT: f32 = 150.0;
const EDITOR_SIDE_PANEL_WIDTH: f32 = 430.0;

#[derive(Resource)]
pub struct SceneEditorState {
    pub enabled: bool,
    pub cursor: IVec2,
    pub cursor_z: i32,
    pub selected_palette_index: usize,
    pub selected_wall_style_index: usize,
    pub selected_tile_kind: TileKindFile,
    pub selected_edge_direction: EdgeDirectionFile,
    pub selected_marker_kind: MarkerKind,
    pub selected_link_kind: VerticalLinkKindFile,
    pub link_bidirectional: bool,
    pub selected_collision_blocked: bool,
    pub tool: EditorTool,
    pub selected: Option<SceneSelection>,
    pub selected_instance_id: Option<String>,
    pub pending_link_start: Option<NavPointFile>,
    pub pending_zone_start: Option<ZoneDraft>,
    pub dirty: bool,
    pub message: String,
}

impl SceneEditorState {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            cursor: IVec2::new(12, 9),
            cursor_z: 0,
            selected_palette_index: 0,
            selected_wall_style_index: 0,
            selected_tile_kind: TileKindFile::TavernFloor,
            selected_edge_direction: EdgeDirectionFile::S,
            selected_marker_kind: MarkerKind::Seat,
            selected_link_kind: VerticalLinkKindFile::Stairs,
            link_bidirectional: true,
            selected_collision_blocked: true,
            tool: EditorTool::Object,
            selected: None,
            selected_instance_id: None,
            pending_link_start: None,
            pending_zone_start: None,
            dirty: false,
            message: String::new(),
        }
    }

    pub fn selected_prefab_id<'a>(&self, catalog: &'a PrefabCatalog) -> Option<&'a str> {
        let palette = catalog.palette_ids();
        if palette.is_empty() {
            None
        } else {
            Some(palette[self.selected_palette_index.min(palette.len() - 1)].as_str())
        }
    }

    pub fn selected_prefab_label<'a>(&self, catalog: &'a PrefabCatalog) -> &'a str {
        self.selected_prefab_id(catalog)
            .map(|id| catalog.label_for(id))
            .unwrap_or("No prefabs")
    }

    pub fn tool_label(&self) -> &'static str {
        self.tool.label()
    }

    pub fn selected_label(&self) -> String {
        self.selected
            .as_ref()
            .map(SceneSelection::label)
            .unwrap_or_else(|| "none".to_string())
    }

    pub fn active_brush_label(&self, document: &WorldDocument, catalog: &PrefabCatalog) -> String {
        match self.tool {
            EditorTool::Object => format!("prefab {}", self.selected_prefab_label(catalog)),
            EditorTool::Tile => format!("tile {}", self.selected_tile_kind.label()),
            EditorTool::EdgeWall => {
                let style = selected_wall_style_name(self, document)
                    .unwrap_or_else(|| "no wall style".to_string());
                format!("edge {} {}", self.selected_edge_direction.label(), style)
            }
            EditorTool::Marker => format!("marker {}", self.selected_marker_kind.label()),
            EditorTool::Link => format!(
                "link {} {}",
                self.selected_link_kind.label(),
                if self.link_bidirectional {
                    "two-way"
                } else {
                    "one-way"
                }
            ),
            EditorTool::Zone => "zone rectangle".to_string(),
            EditorTool::Collision => format!(
                "collision {}",
                if self.selected_collision_blocked {
                    "blocked"
                } else {
                    "clear"
                }
            ),
            EditorTool::World => format!("world {} levels", document.levels),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorTool {
    Object,
    Tile,
    EdgeWall,
    Marker,
    Link,
    Zone,
    Collision,
    World,
}

impl EditorTool {
    fn label(self) -> &'static str {
        match self {
            Self::Object => "Objects",
            Self::Tile => "Tiles",
            Self::EdgeWall => "Edge Walls",
            Self::Marker => "Markers",
            Self::Link => "Vertical Links",
            Self::Zone => "Zones",
            Self::Collision => "Collision",
            Self::World => "World",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerKind {
    Seat,
    Entrance,
}

impl MarkerKind {
    fn label(self) -> &'static str {
        match self {
            Self::Seat => "seat",
            Self::Entrance => "entrance",
        }
    }

    fn cycle(self) -> Self {
        match self {
            Self::Seat => Self::Entrance,
            Self::Entrance => Self::Seat,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneSelection {
    Prefab(String),
    EdgeWall(usize),
    Seat(usize),
    Entrance,
    VerticalLinkEndpoint {
        index: usize,
        endpoint: LinkEndpoint,
    },
    Zone(usize),
    Tile {
        cell: IVec2,
        z: i32,
    },
    CollisionCell {
        cell: IVec2,
        z: i32,
    },
}

impl SceneSelection {
    fn label(&self) -> String {
        match self {
            Self::Prefab(id) => format!("prefab:{id}"),
            Self::EdgeWall(index) => format!("edge-wall:{index}"),
            Self::Seat(index) => format!("seat:{index}"),
            Self::Entrance => "entrance".to_string(),
            Self::VerticalLinkEndpoint { index, endpoint } => {
                format!("link:{index}:{}", endpoint.label())
            }
            Self::Zone(index) => format!("zone:{index}"),
            Self::Tile { cell, z } => format!("tile:{},{}:{z}", cell.x, cell.y),
            Self::CollisionCell { cell, z } => {
                format!("collision:{},{}:{z}", cell.x, cell.y)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkEndpoint {
    Lower,
    Upper,
}

impl LinkEndpoint {
    fn label(self) -> &'static str {
        match self {
            Self::Lower => "lower",
            Self::Upper => "upper",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneDraft {
    pub cell: IVec2,
    pub z: i32,
}

#[derive(Resource)]
pub struct SceneSelectorState {
    pub visible: bool,
    pub scenes: Vec<SceneEntry>,
    revision: u64,
}

impl SceneSelectorState {
    pub fn new(visible: bool) -> Self {
        Self {
            visible,
            scenes: discover_world_scenes(),
            revision: 0,
        }
    }

    fn refresh_scenes(&mut self) {
        self.scenes = discover_world_scenes();
        self.revision = self.revision.saturating_add(1);
    }
}

#[derive(Clone)]
pub struct SceneEntry {
    pub label: String,
    pub path: String,
}

#[derive(Component)]
pub struct EditorUiRoot;

#[derive(Component)]
pub struct SceneSelectorRoot;

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum SceneSelectorAction {
    NewEmpty,
    LoadScene(String),
    Close,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorButtonAction {
    Tool(EditorTool),
    OpenSceneSelector,
    Apply,
    Select,
    Move,
    Duplicate,
    Delete,
    NewEmpty,
    PrevBrush,
    NextBrush,
    Rotate,
    ToggleTwoWay,
    AddLevel,
    RemoveLevel,
    Validate,
    Save,
    Reload,
}

pub fn setup_scene_editor_ui(mut commands: Commands) {
    commands
        .spawn((
            EditorUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(360.0),
                right: Val::Px(430.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(6.0),
                column_gap: Val::Px(6.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.028, 0.028, 0.024, 0.86)),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Scene Editor"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.88, 0.74)),
            ));
            spawn_editor_button_row(
                root,
                &[
                    ("Object", EditorButtonAction::Tool(EditorTool::Object)),
                    ("Tile", EditorButtonAction::Tool(EditorTool::Tile)),
                    ("Wall", EditorButtonAction::Tool(EditorTool::EdgeWall)),
                    ("Marker", EditorButtonAction::Tool(EditorTool::Marker)),
                    ("Link", EditorButtonAction::Tool(EditorTool::Link)),
                    ("Zone", EditorButtonAction::Tool(EditorTool::Zone)),
                    ("Collision", EditorButtonAction::Tool(EditorTool::Collision)),
                    ("World", EditorButtonAction::Tool(EditorTool::World)),
                ],
            );
            spawn_editor_button_row(
                root,
                &[
                    ("Scenes", EditorButtonAction::OpenSceneSelector),
                    ("Apply", EditorButtonAction::Apply),
                    ("Select", EditorButtonAction::Select),
                    ("Move", EditorButtonAction::Move),
                    ("Copy", EditorButtonAction::Duplicate),
                    ("Delete", EditorButtonAction::Delete),
                    ("Empty", EditorButtonAction::NewEmpty),
                ],
            );
            spawn_editor_button_row(
                root,
                &[
                    ("Prev", EditorButtonAction::PrevBrush),
                    ("Next", EditorButtonAction::NextBrush),
                    ("Rotate", EditorButtonAction::Rotate),
                    ("Two-way", EditorButtonAction::ToggleTwoWay),
                    ("+Level", EditorButtonAction::AddLevel),
                    ("-Level", EditorButtonAction::RemoveLevel),
                    ("Check", EditorButtonAction::Validate),
                    ("Save", EditorButtonAction::Save),
                    ("Reload", EditorButtonAction::Reload),
                ],
            );
        });
}

fn spawn_editor_button_row(parent: &mut ChildBuilder, buttons: &[(&str, EditorButtonAction)]) {
    parent
        .spawn(Node {
            column_gap: Val::Px(5.0),
            row_gap: Val::Px(5.0),
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|row| {
            for (label, action) in buttons {
                spawn_editor_button(row, label, *action);
            }
        });
}

fn spawn_editor_button(parent: &mut ChildBuilder, label: &str, action: EditorButtonAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                height: Val::Px(28.0),
                padding: UiRect::horizontal(Val::Px(9.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.15, 0.13, 0.95)),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.86, 0.76)),
        ));
}

pub fn setup_scene_selector_ui(mut commands: Commands, selector: Res<SceneSelectorState>) {
    spawn_scene_selector_panel(&mut commands, &selector);
}

fn spawn_scene_selector_panel(commands: &mut Commands, selector: &SceneSelectorState) {
    commands
        .spawn((
            SceneSelectorRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(96.0),
                width: Val::Px(420.0),
                margin: UiRect::left(Val::Px(-210.0)),
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.018, 0.94)),
            if selector.visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Select Scene"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.88, 0.68)),
            ));
            root.spawn((
                Text::new("Start empty for editing, or deliberately load an existing scene."),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.72, 0.66)),
            ));
            spawn_selector_button(root, "New Empty Scene", SceneSelectorAction::NewEmpty);
            for scene in &selector.scenes {
                spawn_selector_button(
                    root,
                    &format!("Load {}", scene.label),
                    SceneSelectorAction::LoadScene(scene.path.clone()),
                );
            }
            spawn_selector_button(root, "Close Selector", SceneSelectorAction::Close);
        });
}

pub fn sync_scene_selector_ui(
    mut commands: Commands,
    selector: Res<SceneSelectorState>,
    roots: Query<Entity, With<SceneSelectorRoot>>,
    mut last_revision: Local<u64>,
) {
    if *last_revision == selector.revision && !roots.is_empty() {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
    spawn_scene_selector_panel(&mut commands, &selector);
    *last_revision = selector.revision;
}

fn spawn_selector_button(parent: &mut ChildBuilder, label: &str, action: SceneSelectorAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                height: Val::Px(34.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.15, 0.12, 1.0)),
        ))
        .with_child((
            Text::new(label.to_string()),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.92, 0.88, 0.78)),
        ));
}

fn discover_world_scenes() -> Vec<SceneEntry> {
    let worlds_dir = project_path("data/worlds");
    let Ok(entries) = fs::read_dir(&worlds_dir) else {
        return Vec::new();
    };

    let mut scenes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("ron") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let label = file_name.trim_end_matches(".ron").replace('_', " ");
        scenes.push(SceneEntry {
            label,
            path: format!("data/worlds/{file_name}"),
        });
    }

    scenes.sort_by(|left, right| left.label.cmp(&right.label));
    scenes
}

pub fn update_scene_editor_ui_visibility(
    editor: Res<SceneEditorState>,
    selector: Res<SceneSelectorState>,
    mut roots: Query<&mut Visibility, With<EditorUiRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if editor.enabled && !selector.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn update_scene_selector_ui_visibility(
    selector: Res<SceneSelectorState>,
    mut roots: Query<&mut Visibility, With<SceneSelectorRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if selector.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn scene_selector_button_input(
    mut interactions: Query<
        (&Interaction, &SceneSelectorAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<WorldRuntimeConfig>,
    mut document: ResMut<WorldDocument>,
    mut editor: ResMut<SceneEditorState>,
    mut selector: ResMut<SceneSelectorState>,
    mut status: ResMut<WorldRuntimeStatus>,
    mut reload_requests: EventWriter<WorldReloadRequested>,
) {
    for (interaction, action, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.34, 0.29, 0.18, 1.0));
                match action.clone() {
                    SceneSelectorAction::NewEmpty => {
                        selector.visible = false;
                        new_empty_scene(
                            &mut config,
                            &mut editor,
                            &mut document,
                            &mut reload_requests,
                        );
                        status.last_message = editor.message.clone();
                    }
                    SceneSelectorAction::LoadScene(path) => {
                        config.world_path = path.clone();
                        config.start_empty_scene = false;
                        selector.visible = false;
                        editor.dirty = false;
                        editor.message = format!("Loading {path}");
                        status.last_message = editor.message.clone();
                        reload_requests.send(WorldReloadRequested::disk(format!(
                            "scene selector load {path}"
                        )));
                    }
                    SceneSelectorAction::Close => {
                        selector.visible = false;
                        editor.message = "Scene selector closed".to_string();
                        status.last_message = editor.message.clone();
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.24, 0.22, 0.18, 0.98));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.16, 0.15, 0.12, 1.0));
            }
        }
    }
}

pub fn scene_editor_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<WorldRuntimeConfig>,
    asset_registry: Res<AssetRegistry>,
    prefab_catalog: Res<PrefabCatalog>,
    mut document: ResMut<WorldDocument>,
    mut editor: ResMut<SceneEditorState>,
    mut selector: ResMut<SceneSelectorState>,
    mut status: ResMut<WorldRuntimeStatus>,
    mut reload_requests: EventWriter<WorldReloadRequested>,
) {
    if keyboard.just_pressed(KeyCode::F10) {
        editor.enabled = !editor.enabled;
        editor.message = if editor.enabled {
            "Editor enabled".to_string()
        } else {
            selector.visible = false;
            "Editor disabled".to_string()
        };
    }

    if keyboard.just_pressed(KeyCode::F5) {
        reload_requests.send(WorldReloadRequested::disk("manual F5 hot reload"));
        editor.dirty = false;
        return;
    }

    if !editor.enabled {
        return;
    }

    if selector.visible {
        if keyboard.just_pressed(KeyCode::Escape) {
            selector.visible = false;
            editor.message = "Scene selector closed".to_string();
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::F8) {
        validate_editor_document(
            &document,
            &asset_registry,
            &prefab_catalog,
            &mut editor,
            &mut status,
        );
    }

    if keyboard.just_pressed(KeyCode::F9) {
        save_editor_document(
            &mut config,
            &asset_registry,
            &prefab_catalog,
            &document,
            &mut editor,
            &mut status,
        );
    }

    handle_tool_keys(&keyboard, &mut editor);
    handle_cursor_keys(&keyboard, &document, &mut editor);

    if keyboard.just_pressed(KeyCode::Tab) || keyboard.just_pressed(KeyCode::BracketRight) {
        cycle_active_brush(&mut editor, &document, &prefab_catalog, 1);
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        cycle_active_brush(&mut editor, &document, &prefab_catalog, -1);
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        rotate_active_setting(&mut editor, &mut document, &mut reload_requests);
    }
    if keyboard.just_pressed(KeyCode::KeyB) {
        toggle_link_bidirectionality(&mut editor, &mut document, &mut reload_requests);
    }
    if keyboard.just_pressed(KeyCode::KeyY) {
        change_world_levels(1, &mut editor, &mut document, &mut reload_requests);
    }
    if keyboard.just_pressed(KeyCode::KeyU) {
        change_world_levels(-1, &mut editor, &mut document, &mut reload_requests);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        editor.pending_link_start = None;
        editor.pending_zone_start = None;
        editor.message = "Cleared pending editor action".to_string();
    }

    if keyboard.just_pressed(KeyCode::KeyG) {
        select_nearest_for_active_tool(&mut editor, &document);
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        primary_action(
            &mut editor,
            &prefab_catalog,
            &mut document,
            &mut reload_requests,
        );
    }

    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        delete_selected_or_nearest(&mut editor, &mut document, &mut reload_requests);
    }

    if keyboard.just_pressed(KeyCode::KeyM) {
        move_selected_or_nearest(&mut editor, &mut document, &mut reload_requests);
    }

    if keyboard.just_pressed(KeyCode::KeyC) {
        duplicate_selected_or_nearest(&mut editor, &mut document, &mut reload_requests);
    }
}

pub fn scene_editor_button_input(
    mut interactions: Query<
        (&Interaction, &EditorButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<WorldRuntimeConfig>,
    asset_registry: Res<AssetRegistry>,
    prefab_catalog: Res<PrefabCatalog>,
    mut document: ResMut<WorldDocument>,
    mut editor: ResMut<SceneEditorState>,
    mut selector: ResMut<SceneSelectorState>,
    mut status: ResMut<WorldRuntimeStatus>,
    mut reload_requests: EventWriter<WorldReloadRequested>,
) {
    for (interaction, action, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.34, 0.29, 0.18, 1.0));
                run_editor_button_action(
                    *action,
                    &mut config,
                    &asset_registry,
                    &prefab_catalog,
                    &mut document,
                    &mut editor,
                    &mut selector,
                    &mut status,
                    &mut reload_requests,
                );
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.24, 0.22, 0.18, 0.98));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.16, 0.15, 0.13, 0.95));
            }
        }
    }
}

pub fn scene_editor_mouse_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    prefab_catalog: Res<PrefabCatalog>,
    selector: Res<SceneSelectorState>,
    mut document: ResMut<WorldDocument>,
    mut editor: ResMut<SceneEditorState>,
    mut reload_requests: EventWriter<WorldReloadRequested>,
) {
    if !editor.enabled || selector.visible {
        return;
    }
    if !mouse_buttons.just_pressed(MouseButton::Left)
        && !mouse_buttons.just_pressed(MouseButton::Right)
    {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    if cursor_over_editor_ui(cursor_position, window) {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let grid = screen_to_grid_at(world_position, editor.cursor_z).round();
    editor.cursor = IVec2::new(grid.x as i32, grid.y as i32);
    clamp_cursor(&mut editor, &document);

    if mouse_buttons.just_pressed(MouseButton::Left) {
        primary_action(
            &mut editor,
            &prefab_catalog,
            &mut document,
            &mut reload_requests,
        );
    } else if mouse_buttons.just_pressed(MouseButton::Right) {
        select_nearest_for_active_tool(&mut editor, &document);
    }
}

fn run_editor_button_action(
    action: EditorButtonAction,
    config: &mut WorldRuntimeConfig,
    asset_registry: &AssetRegistry,
    prefab_catalog: &PrefabCatalog,
    document: &mut WorldDocument,
    editor: &mut SceneEditorState,
    selector: &mut SceneSelectorState,
    status: &mut WorldRuntimeStatus,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    if !editor.enabled
        && action != EditorButtonAction::Reload
        && action != EditorButtonAction::OpenSceneSelector
    {
        return;
    }

    match action {
        EditorButtonAction::OpenSceneSelector => {
            selector.refresh_scenes();
            selector.visible = true;
            editor.message = "Scene selector opened".to_string();
        }
        EditorButtonAction::Tool(tool) => {
            editor.tool = tool;
            editor.selected = None;
            editor.selected_instance_id = None;
            editor.message = format!("Tool: {}", tool.label());
        }
        EditorButtonAction::Apply => {
            primary_action(editor, prefab_catalog, document, reload_requests);
        }
        EditorButtonAction::Select => {
            select_nearest_for_active_tool(editor, document);
        }
        EditorButtonAction::Move => {
            move_selected_or_nearest(editor, document, reload_requests);
        }
        EditorButtonAction::Duplicate => {
            duplicate_selected_or_nearest(editor, document, reload_requests);
        }
        EditorButtonAction::Delete => {
            delete_selected_or_nearest(editor, document, reload_requests);
        }
        EditorButtonAction::NewEmpty => {
            new_empty_scene(config, editor, document, reload_requests);
        }
        EditorButtonAction::PrevBrush => {
            cycle_active_brush(editor, document, prefab_catalog, -1);
        }
        EditorButtonAction::NextBrush => {
            cycle_active_brush(editor, document, prefab_catalog, 1);
        }
        EditorButtonAction::Rotate => {
            rotate_active_setting(editor, document, reload_requests);
        }
        EditorButtonAction::ToggleTwoWay => {
            toggle_link_bidirectionality(editor, document, reload_requests);
        }
        EditorButtonAction::AddLevel => {
            change_world_levels(1, editor, document, reload_requests);
        }
        EditorButtonAction::RemoveLevel => {
            change_world_levels(-1, editor, document, reload_requests);
        }
        EditorButtonAction::Validate => {
            validate_editor_document(document, asset_registry, prefab_catalog, editor, status);
        }
        EditorButtonAction::Save => {
            save_editor_document(
                config,
                asset_registry,
                prefab_catalog,
                document,
                editor,
                status,
            );
        }
        EditorButtonAction::Reload => {
            reload_requests.send(WorldReloadRequested::disk("editor button reload"));
            editor.dirty = false;
        }
    }
}

pub fn draw_scene_editor(
    mut gizmos: Gizmos,
    editor: Res<SceneEditorState>,
    selector: Res<SceneSelectorState>,
    document: Res<WorldDocument>,
    objects: Query<(&WorldObjectEntity, &GridPosition)>,
) {
    if !editor.enabled || selector.visible {
        return;
    }

    draw_editor_cell(
        &mut gizmos,
        Vec2::new(editor.cursor.x as f32, editor.cursor.y as f32),
        editor.cursor_z,
        Color::srgba(1.0, 0.86, 0.24, 0.95),
    );

    draw_prefab_handles(&mut gizmos, &editor, &document, &objects);
    draw_edge_wall_handles(&mut gizmos, &editor, &document);
    draw_marker_handles(&mut gizmos, &editor, &document);
    draw_vertical_link_handles(&mut gizmos, &editor, &document);
    draw_zone_handles(&mut gizmos, &editor, &document);
    draw_collision_handles(&mut gizmos, &editor, &document);
}

fn handle_tool_keys(keyboard: &ButtonInput<KeyCode>, editor: &mut SceneEditorState) {
    let tool = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(EditorTool::Object)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(EditorTool::Tile)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(EditorTool::EdgeWall)
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(EditorTool::Marker)
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(EditorTool::Link)
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        Some(EditorTool::Zone)
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        Some(EditorTool::Collision)
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        Some(EditorTool::World)
    } else {
        None
    };

    if let Some(tool) = tool {
        editor.tool = tool;
        editor.selected = None;
        editor.selected_instance_id = None;
        editor.message = format!("Tool: {}", tool.label());
    }
}

fn handle_cursor_keys(
    keyboard: &ButtonInput<KeyCode>,
    document: &WorldDocument,
    editor: &mut SceneEditorState,
) {
    let previous_cursor = editor.cursor;
    let previous_cursor_z = editor.cursor_z;

    if keyboard.just_pressed(KeyCode::KeyI) {
        editor.cursor.y -= 1;
    }
    if keyboard.just_pressed(KeyCode::KeyK) {
        editor.cursor.y += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyJ) {
        editor.cursor.x -= 1;
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        editor.cursor.x += 1;
    }
    clamp_cursor(editor, document);

    if keyboard.just_pressed(KeyCode::PageUp) {
        editor.cursor_z += 1;
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        editor.cursor_z -= 1;
    }
    clamp_cursor(editor, document);

    if previous_cursor != editor.cursor || previous_cursor_z != editor.cursor_z {
        editor.message = format!(
            "Cursor {},{},{}",
            editor.cursor.x, editor.cursor.y, editor.cursor_z
        );
    }
}

fn clamp_cursor(editor: &mut SceneEditorState, document: &WorldDocument) {
    editor.cursor.x = editor.cursor.x.clamp(0, document.width.saturating_sub(1));
    editor.cursor.y = editor.cursor.y.clamp(0, document.height.saturating_sub(1));
    editor.cursor_z = editor.cursor_z.clamp(0, document.levels.saturating_sub(1));
}

fn cursor_over_editor_ui(cursor_position: Vec2, window: &Window) -> bool {
    cursor_position.y <= EDITOR_TOP_BAR_HEIGHT
        || cursor_position.y >= window.height() - EDITOR_TOP_BAR_HEIGHT
        || cursor_position.x >= window.width() - EDITOR_SIDE_PANEL_WIDTH
}

fn validate_editor_document(
    document: &WorldDocument,
    asset_registry: &AssetRegistry,
    prefab_catalog: &PrefabCatalog,
    editor: &mut SceneEditorState,
    status: &mut WorldRuntimeStatus,
) {
    match document
        .clone()
        .into_blueprint(asset_registry, prefab_catalog)
    {
        Ok(blueprint) => {
            editor.message = format!(
                "Validation ok: {} tiles, {} props, {} edge walls, {} links",
                blueprint.tiles.len(),
                blueprint.props.len() + blueprint.stations.len(),
                blueprint.edge_walls.len(),
                blueprint.vertical_links.len()
            );
            status.last_message = editor.message.clone();
        }
        Err(error) => {
            editor.message = format!("Validation failed: {error}");
            status.last_message = editor.message.clone();
        }
    }
}

fn save_editor_document(
    config: &mut WorldRuntimeConfig,
    asset_registry: &AssetRegistry,
    prefab_catalog: &PrefabCatalog,
    document: &WorldDocument,
    editor: &mut SceneEditorState,
    status: &mut WorldRuntimeStatus,
) {
    if let Err(error) = document
        .clone()
        .into_blueprint(asset_registry, prefab_catalog)
    {
        editor.message = format!("Save blocked: {error}");
        status.last_message = editor.message.clone();
        return;
    }

    match save_current_world_document(config, document) {
        Ok(()) => {
            config.start_empty_scene = false;
            editor.dirty = false;
            editor.message = format!("Saved {}", config.world_path);
            status.last_message = editor.message.clone();
        }
        Err(error) => {
            editor.message = format!("Save failed: {error}");
            status.last_message = editor.message.clone();
        }
    }
}

fn cycle_active_brush(
    editor: &mut SceneEditorState,
    document: &WorldDocument,
    catalog: &PrefabCatalog,
    step: isize,
) {
    match editor.tool {
        EditorTool::Object => cycle_palette(editor, catalog, step),
        EditorTool::Tile => {
            editor.selected_tile_kind = editor.selected_tile_kind.cycle(step);
            editor.message = format!("Tile: {}", editor.selected_tile_kind.label());
        }
        EditorTool::EdgeWall => {
            let len = wall_style_names(document).len();
            if len == 0 {
                editor.message = "No wall styles in world document".to_string();
            } else {
                let current = editor.selected_wall_style_index.min(len - 1) as isize;
                editor.selected_wall_style_index =
                    (current + step).rem_euclid(len as isize) as usize;
                editor.message = format!(
                    "Wall style: {}",
                    selected_wall_style_name(editor, document).unwrap_or_default()
                );
            }
        }
        EditorTool::Marker => {
            editor.selected_marker_kind = editor.selected_marker_kind.cycle();
            editor.message = format!("Marker: {}", editor.selected_marker_kind.label());
        }
        EditorTool::Link => {
            editor.selected_link_kind = editor.selected_link_kind.cycle(step);
            editor.message = format!("Link kind: {}", editor.selected_link_kind.label());
        }
        EditorTool::Collision => {
            editor.selected_collision_blocked = !editor.selected_collision_blocked;
            editor.message = format!(
                "Collision brush: {}",
                if editor.selected_collision_blocked {
                    "blocked"
                } else {
                    "clear"
                }
            );
        }
        EditorTool::Zone | EditorTool::World => {
            editor.message = "No brush palette for this tool".to_string();
        }
    }
}

fn rotate_active_setting(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    match editor.tool {
        EditorTool::EdgeWall => {
            if let Some(index) = selected_edge_wall_index(editor, document) {
                document.edge_walls[index].direction =
                    document.edge_walls[index].direction.cycle(1);
                editor.message = format!(
                    "Rotated edge wall to {}",
                    document.edge_walls[index].direction.label()
                );
                mark_document_changed(editor, reload_requests, "editor rotate edge wall");
            } else {
                editor.selected_edge_direction = editor.selected_edge_direction.cycle(1);
                editor.message =
                    format!("Edge direction: {}", editor.selected_edge_direction.label());
            }
        }
        EditorTool::Link => {
            if let Some(index) = selected_link_index(editor, document) {
                document.vertical_links[index].kind = document.vertical_links[index].kind.cycle(1);
                editor.message =
                    format!("Link kind: {}", document.vertical_links[index].kind.label());
                mark_document_changed(editor, reload_requests, "editor link kind");
            } else {
                editor.selected_link_kind = editor.selected_link_kind.cycle(1);
                editor.message = format!("Link kind: {}", editor.selected_link_kind.label());
            }
        }
        _ => {
            editor.message = "Rotate applies to edge walls and links".to_string();
        }
    }
}

fn toggle_link_bidirectionality(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    if let Some(index) = selected_link_index(editor, document) {
        document.vertical_links[index].bidirectional =
            !document.vertical_links[index].bidirectional;
        editor.message = format!(
            "Link {}",
            if document.vertical_links[index].bidirectional {
                "two-way"
            } else {
                "one-way"
            }
        );
        mark_document_changed(editor, reload_requests, "editor toggle link direction");
    } else {
        editor.link_bidirectional = !editor.link_bidirectional;
        editor.message = format!(
            "New links {}",
            if editor.link_bidirectional {
                "two-way"
            } else {
                "one-way"
            }
        );
    }
}

fn change_world_levels(
    delta: i32,
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    if delta > 0 {
        document.levels += 1;
        editor.message = format!("World levels: {}", document.levels);
        mark_document_changed(editor, reload_requests, "editor add level");
        return;
    }

    if document.levels <= 1 {
        editor.message = "World already has one level".to_string();
        return;
    }

    let new_top = document.levels - 2;
    if max_used_z(document) > new_top {
        editor.message = "Cannot remove top level while content exists there".to_string();
        return;
    }

    document.levels -= 1;
    editor.cursor_z = editor.cursor_z.min(document.levels - 1);
    editor.message = format!("World levels: {}", document.levels);
    mark_document_changed(editor, reload_requests, "editor remove level");
}

fn new_empty_scene(
    config: &mut WorldRuntimeConfig,
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    config.world_path = UNTITLED_SCENE_PATH.to_string();
    config.start_empty_scene = true;
    let prop_styles = document.prop_styles.clone();
    let mut empty = WorldDocument::empty_scene(document.width, document.height, document.levels);
    empty.prop_styles = prop_styles;
    *document = empty;
    editor.cursor = IVec2::ZERO;
    editor.cursor_z = 0;
    editor.selected = None;
    editor.selected_instance_id = None;
    editor.pending_link_start = None;
    editor.pending_zone_start = None;
    editor.message = format!("Created empty scene document: {}", config.world_path);
    mark_document_changed(editor, reload_requests, "editor new empty scene");
}

fn primary_action(
    editor: &mut SceneEditorState,
    catalog: &PrefabCatalog,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    match editor.tool {
        EditorTool::Object => place_selected_prefab(editor, catalog, document, reload_requests),
        EditorTool::Tile => paint_tile_at_cursor(editor, document, reload_requests),
        EditorTool::EdgeWall => upsert_edge_wall_at_cursor(editor, document, reload_requests),
        EditorTool::Marker => place_marker_at_cursor(editor, document, reload_requests),
        EditorTool::Link => edit_vertical_link_at_cursor(editor, document, reload_requests),
        EditorTool::Zone => edit_zone_at_cursor(editor, document, reload_requests),
        EditorTool::Collision => paint_collision_at_cursor(editor, document, reload_requests),
        EditorTool::World => editor.message = "World tool: Y add level, U remove level".to_string(),
    }
}

fn cycle_palette(editor: &mut SceneEditorState, catalog: &PrefabCatalog, step: isize) {
    let len = catalog.palette_ids().len();
    if len == 0 {
        editor.message = "No prefabs in palette".to_string();
        return;
    }

    let current = editor.selected_palette_index.min(len - 1) as isize;
    editor.selected_palette_index = (current + step).rem_euclid(len as isize) as usize;
    editor.message = format!("Prefab: {}", editor.selected_prefab_label(catalog));
}

fn place_selected_prefab(
    editor: &mut SceneEditorState,
    catalog: &PrefabCatalog,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    let Some(prefab_id) = editor.selected_prefab_id(catalog).map(str::to_string) else {
        editor.message = "No prefab selected".to_string();
        return;
    };

    let grid = cursor_grid(editor);
    let id = document.next_prefab_instance_id(&prefab_id);
    document.prefab_instances.push(PrefabInstanceFile::at(
        id.clone(),
        prefab_id,
        grid,
        editor.cursor_z,
    ));
    select_prefab(editor, id.clone());
    editor.message = format!("Placed {id}");
    mark_document_changed(editor, reload_requests, "editor place prefab");
}

fn paint_tile_at_cursor(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    paint_tile_cell(
        document,
        editor.cursor,
        editor.cursor_z,
        editor.selected_tile_kind,
    );
    editor.selected = Some(SceneSelection::Tile {
        cell: editor.cursor,
        z: editor.cursor_z,
    });
    editor.message = format!(
        "Painted {},{},{} as {}",
        editor.cursor.x,
        editor.cursor.y,
        editor.cursor_z,
        editor.selected_tile_kind.label()
    );
    mark_document_changed(editor, reload_requests, "editor paint tile");
}

fn upsert_edge_wall_at_cursor(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    let Some(style) = selected_wall_style_name(editor, document) else {
        editor.message = "No wall prop style available".to_string();
        return;
    };

    let index = upsert_edge_wall(
        document,
        editor.cursor,
        editor.cursor_z,
        editor.selected_edge_direction,
        style.clone(),
    );
    editor.selected = Some(SceneSelection::EdgeWall(index));
    editor.selected_instance_id = None;
    editor.message = format!(
        "Edge wall {},{},{} {} style {}",
        editor.cursor.x,
        editor.cursor.y,
        editor.cursor_z,
        editor.selected_edge_direction.label(),
        style
    );
    mark_document_changed(editor, reload_requests, "editor edge wall");
}

fn place_marker_at_cursor(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    match editor.selected_marker_kind {
        MarkerKind::Seat => {
            document
                .seats
                .push(NavPointFile::at(cursor_grid(editor), editor.cursor_z));
            let index = document.seats.len() - 1;
            editor.selected = Some(SceneSelection::Seat(index));
            editor.message = format!("Placed seat marker {index}");
        }
        MarkerKind::Entrance => {
            document.entrance.set(cursor_grid(editor), editor.cursor_z);
            editor.selected = Some(SceneSelection::Entrance);
            editor.message = "Moved entrance marker".to_string();
        }
    }
    editor.selected_instance_id = None;
    mark_document_changed(editor, reload_requests, "editor marker");
}

fn edit_vertical_link_at_cursor(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    let endpoint = NavPointFile::at(cursor_grid(editor), editor.cursor_z);
    let Some(start) = editor.pending_link_start.take() else {
        editor.pending_link_start = Some(endpoint);
        editor.message =
            "Link start set; move cursor and press Enter for the other end".to_string();
        return;
    };

    match create_vertical_link(
        document,
        start,
        endpoint,
        editor.selected_link_kind,
        editor.link_bidirectional,
    ) {
        Ok(index) => {
            editor.selected = Some(SceneSelection::VerticalLinkEndpoint {
                index,
                endpoint: LinkEndpoint::Upper,
            });
            editor.selected_instance_id = None;
            editor.message = format!("Created {} link {index}", editor.selected_link_kind.label());
            mark_document_changed(editor, reload_requests, "editor vertical link");
        }
        Err(error) => {
            editor.message = error;
        }
    }
}

fn edit_zone_at_cursor(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    let endpoint = ZoneDraft {
        cell: editor.cursor,
        z: editor.cursor_z,
    };
    let Some(start) = editor.pending_zone_start.take() else {
        editor.pending_zone_start = Some(endpoint);
        editor.message =
            "Zone start set; move cursor and press Enter for opposite corner".to_string();
        return;
    };

    let index = create_zone(document, start, endpoint);
    editor.selected = Some(SceneSelection::Zone(index));
    editor.selected_instance_id = None;
    editor.message = format!("Created zone {}", document.zones[index].name);
    mark_document_changed(editor, reload_requests, "editor zone");
}

fn paint_collision_at_cursor(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    upsert_collision_cell(
        document,
        editor.cursor,
        editor.cursor_z,
        editor.selected_collision_blocked,
    );
    editor.selected = Some(SceneSelection::CollisionCell {
        cell: editor.cursor,
        z: editor.cursor_z,
    });
    editor.selected_instance_id = None;
    editor.message = format!(
        "Collision {},{},{} -> {}",
        editor.cursor.x,
        editor.cursor.y,
        editor.cursor_z,
        if editor.selected_collision_blocked {
            "blocked"
        } else {
            "clear"
        }
    );
    mark_document_changed(editor, reload_requests, "editor collision cell");
}

fn delete_selected_or_nearest(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    ensure_selection_for_active_tool(editor, document);

    let Some(selection) = editor.selected.clone() else {
        editor.message = "Nothing selected for delete".to_string();
        return;
    };

    match selection {
        SceneSelection::Prefab(id) => {
            if let Some(index) = document
                .prefab_instances
                .iter()
                .position(|instance| instance.id == id)
            {
                document.prefab_instances.remove(index);
                editor.selected_instance_id = None;
                editor.selected = None;
                editor.message = format!("Deleted prefab {id}");
                mark_document_changed(editor, reload_requests, "editor delete prefab");
            }
        }
        SceneSelection::EdgeWall(index) if index < document.edge_walls.len() => {
            document.edge_walls.remove(index);
            editor.selected = None;
            editor.message = format!("Deleted edge wall {index}");
            mark_document_changed(editor, reload_requests, "editor delete edge wall");
        }
        SceneSelection::Seat(index) if index < document.seats.len() => {
            document.seats.remove(index);
            editor.selected = None;
            editor.message = format!("Deleted seat {index}");
            mark_document_changed(editor, reload_requests, "editor delete seat");
        }
        SceneSelection::Entrance => {
            editor.message = "Entrance cannot be deleted; move it instead".to_string();
        }
        SceneSelection::VerticalLinkEndpoint { index, .. }
            if index < document.vertical_links.len() =>
        {
            document.vertical_links.remove(index);
            editor.selected = None;
            editor.message = format!("Deleted vertical link {index}");
            mark_document_changed(editor, reload_requests, "editor delete link");
        }
        SceneSelection::Zone(index) if index < document.zones.len() => {
            let name = document.zones[index].name.clone();
            document.zones.remove(index);
            editor.selected = None;
            editor.message = format!("Deleted zone {name}");
            mark_document_changed(editor, reload_requests, "editor delete zone");
        }
        SceneSelection::Tile { .. } => {
            paint_tile_cell(
                document,
                editor.cursor,
                editor.cursor_z,
                TileKindFile::Ground,
            );
            editor.message = "Painted tile back to ground".to_string();
            mark_document_changed(editor, reload_requests, "editor clear tile");
        }
        SceneSelection::CollisionCell { cell, z } => {
            if let Some(index) = document
                .collision_cells
                .iter()
                .position(|collision| collision.grid == vec![cell.x, cell.y] && collision.z == z)
            {
                document.collision_cells.remove(index);
                editor.selected = None;
                editor.message = "Removed collision override".to_string();
                mark_document_changed(editor, reload_requests, "editor delete collision");
            }
        }
        _ => editor.message = "Selected item no longer exists".to_string(),
    }
}

fn move_selected_or_nearest(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    ensure_selection_for_active_tool(editor, document);

    let Some(selection) = editor.selected.clone() else {
        editor.message = "Nothing selected for move".to_string();
        return;
    };

    match selection {
        SceneSelection::Prefab(id) => {
            if let Some(index) = document
                .prefab_instances
                .iter_mut()
                .position(|instance| instance.id == id)
            {
                document.prefab_instances[index].set_grid(cursor_grid(editor));
                document.prefab_instances[index].set_level(editor.cursor_z);
                editor.message = format!("Moved prefab {id}");
                mark_document_changed(editor, reload_requests, "editor move prefab");
            }
        }
        SceneSelection::EdgeWall(index) if index < document.edge_walls.len() => {
            document.edge_walls[index].grid = vec![editor.cursor.x, editor.cursor.y];
            document.edge_walls[index].z = editor.cursor_z;
            editor.message = format!("Moved edge wall {index}");
            mark_document_changed(editor, reload_requests, "editor move edge wall");
        }
        SceneSelection::Seat(index) if index < document.seats.len() => {
            document.seats[index].set(cursor_grid(editor), editor.cursor_z);
            editor.message = format!("Moved seat {index}");
            mark_document_changed(editor, reload_requests, "editor move seat");
        }
        SceneSelection::Entrance => {
            document.entrance.set(cursor_grid(editor), editor.cursor_z);
            editor.message = "Moved entrance".to_string();
            mark_document_changed(editor, reload_requests, "editor move entrance");
        }
        SceneSelection::VerticalLinkEndpoint { index, endpoint }
            if index < document.vertical_links.len() =>
        {
            if move_link_endpoint(document, index, endpoint, cursor_nav_point(editor)).is_ok() {
                editor.message = format!("Moved link {index} {}", endpoint.label());
                mark_document_changed(editor, reload_requests, "editor move link endpoint");
            } else {
                editor.message = "Link endpoints must live on different z levels".to_string();
            }
        }
        SceneSelection::Zone(index) if index < document.zones.len() => {
            move_zone_to_cursor(&mut document.zones[index], editor.cursor, editor.cursor_z);
            editor.message = format!("Moved zone {}", document.zones[index].name);
            mark_document_changed(editor, reload_requests, "editor move zone");
        }
        SceneSelection::Tile { .. } => {
            editor.message = "Tiles are painted with Enter, not moved".to_string();
        }
        SceneSelection::CollisionCell { .. } => {
            editor.message = "Collision cells are painted with Enter, not moved".to_string();
        }
        _ => editor.message = "Selected item no longer exists".to_string(),
    }
}

fn duplicate_selected_or_nearest(
    editor: &mut SceneEditorState,
    document: &mut WorldDocument,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
) {
    ensure_selection_for_active_tool(editor, document);

    let Some(selection) = editor.selected.clone() else {
        editor.message = "Nothing selected for duplicate".to_string();
        return;
    };

    match selection {
        SceneSelection::Prefab(id) => {
            if let Some(index) = document
                .prefab_instances
                .iter()
                .position(|instance| instance.id == id)
            {
                let mut copy = document.prefab_instances[index].clone();
                copy.id = document.next_prefab_instance_id(&copy.prefab);
                copy.set_grid(cursor_grid(editor));
                copy.set_level(editor.cursor_z);
                let id = copy.id.clone();
                document.prefab_instances.push(copy);
                select_prefab(editor, id.clone());
                editor.message = format!("Duplicated prefab {id}");
                mark_document_changed(editor, reload_requests, "editor duplicate prefab");
            }
        }
        SceneSelection::EdgeWall(index) if index < document.edge_walls.len() => {
            let mut copy = document.edge_walls[index].clone();
            copy.grid = vec![editor.cursor.x, editor.cursor.y];
            copy.z = editor.cursor_z;
            document.edge_walls.push(copy);
            editor.selected = Some(SceneSelection::EdgeWall(document.edge_walls.len() - 1));
            editor.message = "Duplicated edge wall".to_string();
            mark_document_changed(editor, reload_requests, "editor duplicate edge wall");
        }
        SceneSelection::Seat(index) if index < document.seats.len() => {
            let mut copy = document.seats[index].clone();
            copy.set(cursor_grid(editor), editor.cursor_z);
            document.seats.push(copy);
            editor.selected = Some(SceneSelection::Seat(document.seats.len() - 1));
            editor.message = "Duplicated seat".to_string();
            mark_document_changed(editor, reload_requests, "editor duplicate seat");
        }
        SceneSelection::VerticalLinkEndpoint { index, .. }
            if index < document.vertical_links.len() =>
        {
            let mut copy = document.vertical_links[index].clone();
            copy.id = next_vertical_link_id(document, copy.kind);
            document.vertical_links.push(copy);
            editor.selected = Some(SceneSelection::VerticalLinkEndpoint {
                index: document.vertical_links.len() - 1,
                endpoint: LinkEndpoint::Lower,
            });
            editor.message = "Duplicated vertical link".to_string();
            mark_document_changed(editor, reload_requests, "editor duplicate link");
        }
        SceneSelection::Zone(index) if index < document.zones.len() => {
            let mut copy = document.zones[index].clone();
            copy.name = next_zone_name(document);
            document.zones.push(copy);
            editor.selected = Some(SceneSelection::Zone(document.zones.len() - 1));
            editor.message = "Duplicated zone".to_string();
            mark_document_changed(editor, reload_requests, "editor duplicate zone");
        }
        SceneSelection::CollisionCell { .. } => {
            paint_collision_at_cursor(editor, document, reload_requests);
        }
        _ => editor.message = "This selection cannot be duplicated".to_string(),
    }
}

fn select_nearest_for_active_tool(editor: &mut SceneEditorState, document: &WorldDocument) {
    editor.selected = match editor.tool {
        EditorTool::Object => nearest_instance_index(document, editor.cursor, editor.cursor_z)
            .map(|index| SceneSelection::Prefab(document.prefab_instances[index].id.clone())),
        EditorTool::Tile => Some(SceneSelection::Tile {
            cell: editor.cursor,
            z: editor.cursor_z,
        }),
        EditorTool::EdgeWall => nearest_edge_wall_index(document, editor.cursor, editor.cursor_z)
            .map(SceneSelection::EdgeWall),
        EditorTool::Marker => nearest_marker_selection(document, editor),
        EditorTool::Link => nearest_link_endpoint_selection(document, editor),
        EditorTool::Zone => zone_at_cursor(document, editor).map(SceneSelection::Zone),
        EditorTool::Collision => Some(SceneSelection::CollisionCell {
            cell: editor.cursor,
            z: editor.cursor_z,
        }),
        EditorTool::World => None,
    };

    if let Some(SceneSelection::Prefab(id)) = &editor.selected {
        editor.selected_instance_id = Some(id.clone());
    } else {
        editor.selected_instance_id = None;
    }

    editor.message = editor
        .selected
        .as_ref()
        .map(|selection| format!("Selected {}", selection.label()))
        .unwrap_or_else(|| "No scene item near cursor".to_string());
}

fn ensure_selection_for_active_tool(editor: &mut SceneEditorState, document: &WorldDocument) {
    if editor.selected.is_none() {
        select_nearest_for_active_tool(editor, document);
    }
}

fn select_prefab(editor: &mut SceneEditorState, id: String) {
    editor.selected_instance_id = Some(id.clone());
    editor.selected = Some(SceneSelection::Prefab(id));
}

fn selected_edge_wall_index(editor: &SceneEditorState, document: &WorldDocument) -> Option<usize> {
    match editor.selected {
        Some(SceneSelection::EdgeWall(index)) if index < document.edge_walls.len() => Some(index),
        _ => nearest_edge_wall_index(document, editor.cursor, editor.cursor_z),
    }
}

fn selected_link_index(editor: &SceneEditorState, document: &WorldDocument) -> Option<usize> {
    match editor.selected {
        Some(SceneSelection::VerticalLinkEndpoint { index, .. })
            if index < document.vertical_links.len() =>
        {
            Some(index)
        }
        _ => None,
    }
}

fn nearest_instance_index(document: &WorldDocument, cursor: IVec2, z: i32) -> Option<usize> {
    let cursor = Vec2::new(cursor.x as f32, cursor.y as f32);
    document
        .prefab_instances
        .iter()
        .enumerate()
        .filter(|(_, instance)| instance.z == z)
        .filter_map(|(index, instance)| {
            let grid = instance.grid_vec().ok()?;
            Some((index, grid.distance(cursor)))
        })
        .filter(|(_, distance)| *distance <= 0.85)
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

fn nearest_edge_wall_index(document: &WorldDocument, cursor: IVec2, z: i32) -> Option<usize> {
    document
        .edge_walls
        .iter()
        .enumerate()
        .filter(|(_, wall)| wall.z == z)
        .filter_map(|(index, wall)| {
            let grid = ivec2_from_slice(&wall.grid)?;
            Some((index, grid.as_vec2().distance(cursor.as_vec2())))
        })
        .filter(|(_, distance)| *distance <= 1.0)
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

fn nearest_marker_selection(
    document: &WorldDocument,
    editor: &SceneEditorState,
) -> Option<SceneSelection> {
    if editor.selected_marker_kind == MarkerKind::Entrance {
        return Some(SceneSelection::Entrance);
    }

    let cursor = cursor_grid(editor);
    document
        .seats
        .iter()
        .enumerate()
        .filter(|(_, seat)| seat.z() == editor.cursor_z)
        .filter_map(|(index, seat)| Some((index, seat.grid_vec().ok()?.distance(cursor))))
        .filter(|(_, distance)| *distance <= 1.0)
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| SceneSelection::Seat(index))
}

fn nearest_link_endpoint_selection(
    document: &WorldDocument,
    editor: &SceneEditorState,
) -> Option<SceneSelection> {
    let cursor = cursor_grid(editor);
    document
        .vertical_links
        .iter()
        .enumerate()
        .flat_map(|(index, link)| {
            [
                (index, LinkEndpoint::Lower, &link.lower),
                (index, LinkEndpoint::Upper, &link.upper),
            ]
        })
        .filter(|(_, _, point)| point.z() == editor.cursor_z)
        .filter_map(|(index, endpoint, point)| {
            Some((index, endpoint, point.grid_vec().ok()?.distance(cursor)))
        })
        .filter(|(_, _, distance)| *distance <= 1.0)
        .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b))
        .map(|(index, endpoint, _)| SceneSelection::VerticalLinkEndpoint { index, endpoint })
}

fn zone_at_cursor(document: &WorldDocument, editor: &SceneEditorState) -> Option<usize> {
    let cursor = cursor_grid(editor);
    document.zones.iter().position(|zone| {
        let Ok(min) = vec2_from_slice(&zone.min) else {
            return false;
        };
        let Ok(max) = vec2_from_slice(&zone.max) else {
            return false;
        };
        cursor.x >= min.x
            && cursor.x <= max.x
            && cursor.y >= min.y
            && cursor.y <= max.y
            && editor.cursor_z >= zone.min_z
            && editor.cursor_z <= zone.max_z
    })
}

fn paint_tile_cell(document: &mut WorldDocument, cell: IVec2, z: i32, kind: TileKindFile) {
    let from = vec![cell.x, cell.y];
    if let Some(region) = document
        .tile_regions
        .iter_mut()
        .find(|region| region.z == z && region.from == from && region.to == from)
    {
        region.kind = kind;
        return;
    }

    document.tile_regions.push(TileRegionFile {
        kind,
        z,
        from: from.clone(),
        to: from,
    });
}

fn upsert_edge_wall(
    document: &mut WorldDocument,
    cell: IVec2,
    z: i32,
    direction: EdgeDirectionFile,
    style: String,
) -> usize {
    if let Some(index) = document.edge_walls.iter().position(|wall| {
        wall.z == z && wall.direction == direction && wall.grid == vec![cell.x, cell.y]
    }) {
        document.edge_walls[index].style = PropStyleRefFile::Named(style);
        return index;
    }

    document.edge_walls.push(EdgeWallFile {
        grid: vec![cell.x, cell.y],
        z,
        direction,
        style: PropStyleRefFile::Named(style),
    });
    document.edge_walls.len() - 1
}

fn upsert_collision_cell(
    document: &mut WorldDocument,
    cell: IVec2,
    z: i32,
    blocked: bool,
) -> usize {
    if let Some(index) = document
        .collision_cells
        .iter()
        .position(|collision| collision.grid == vec![cell.x, cell.y] && collision.z == z)
    {
        document.collision_cells[index].blocked = blocked;
        return index;
    }

    document
        .collision_cells
        .push(CollisionCellFile::at(cell, z, blocked));
    document.collision_cells.len() - 1
}

fn create_vertical_link(
    document: &mut WorldDocument,
    first: NavPointFile,
    second: NavPointFile,
    kind: VerticalLinkKindFile,
    bidirectional: bool,
) -> Result<usize, String> {
    if first.z() == second.z() {
        return Err("Vertical link endpoints must be on different z levels".to_string());
    }

    let (lower, upper) = if first.z() <= second.z() {
        (first, second)
    } else {
        (second, first)
    };
    let id = next_vertical_link_id(document, kind);
    document.vertical_links.push(VerticalLinkFile {
        id,
        kind,
        lower,
        upper,
        bidirectional,
    });
    Ok(document.vertical_links.len() - 1)
}

fn move_link_endpoint(
    document: &mut WorldDocument,
    index: usize,
    endpoint: LinkEndpoint,
    point: NavPointFile,
) -> Result<(), String> {
    let link = &mut document.vertical_links[index];
    match endpoint {
        LinkEndpoint::Lower if point.z() != link.upper.z() => link.lower = point,
        LinkEndpoint::Upper if point.z() != link.lower.z() => link.upper = point,
        _ => return Err("matching z".to_string()),
    }
    Ok(())
}

fn create_zone(document: &mut WorldDocument, first: ZoneDraft, second: ZoneDraft) -> usize {
    let min_x = first.cell.x.min(second.cell.x) as f32;
    let min_y = first.cell.y.min(second.cell.y) as f32;
    let max_x = first.cell.x.max(second.cell.x) as f32;
    let max_y = first.cell.y.max(second.cell.y) as f32;
    let min_z = first.z.min(second.z);
    let max_z = first.z.max(second.z);
    let name = next_zone_name(document);

    document.zones.push(ZoneFile {
        name,
        min: vec![min_x, min_y],
        max: vec![max_x, max_y],
        min_z,
        max_z,
    });
    document.zones.len() - 1
}

fn move_zone_to_cursor(zone: &mut ZoneFile, cell: IVec2, z: i32) {
    let min = vec2_from_slice(&zone.min).unwrap_or(Vec2::ZERO);
    let max = vec2_from_slice(&zone.max).unwrap_or(min);
    let size = max - min;
    let z_span = zone.max_z - zone.min_z;
    zone.min = vec![cell.x as f32, cell.y as f32];
    zone.max = vec![cell.x as f32 + size.x, cell.y as f32 + size.y];
    zone.min_z = z;
    zone.max_z = z + z_span;
}

fn wall_style_names(document: &WorldDocument) -> Vec<String> {
    let wall_styles: Vec<String> = document
        .prop_styles
        .iter()
        .filter(|definition| matches!(definition.style.layer, PropLayerFile::Wall))
        .map(|definition| definition.name.clone())
        .collect();

    if wall_styles.is_empty() {
        document
            .prop_styles
            .iter()
            .map(|definition| definition.name.clone())
            .collect()
    } else {
        wall_styles
    }
}

fn selected_wall_style_name(editor: &SceneEditorState, document: &WorldDocument) -> Option<String> {
    let names = wall_style_names(document);
    if names.is_empty() {
        None
    } else {
        Some(names[editor.selected_wall_style_index.min(names.len() - 1)].clone())
    }
}

fn next_vertical_link_id(document: &WorldDocument, kind: VerticalLinkKindFile) -> String {
    let prefix = kind.label().replace(' ', "_");
    let mut index = document.vertical_links.len() + 1;
    loop {
        let candidate = format!("{prefix}_{index:03}");
        if !document
            .vertical_links
            .iter()
            .any(|link| link.id == candidate)
        {
            return candidate;
        }
        index += 1;
    }
}

fn next_zone_name(document: &WorldDocument) -> String {
    let mut index = document.zones.len() + 1;
    loop {
        let candidate = format!("zone_{index:03}");
        if !document.zones.iter().any(|zone| zone.name == candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn max_used_z(document: &WorldDocument) -> i32 {
    let mut max_z = 0;
    for region in &document.tile_regions {
        max_z = max_z.max(region.z);
    }
    for collision in &document.collision_cells {
        max_z = max_z.max(collision.z);
    }
    for prop in &document.props {
        max_z = max_z.max(prop.z);
    }
    for run in &document.prop_runs {
        max_z = max_z.max(run.z);
    }
    for wall in &document.edge_walls {
        max_z = max_z.max(wall.z);
    }
    for station in &document.stations {
        max_z = max_z.max(station.z);
    }
    for instance in &document.prefab_instances {
        max_z = max_z.max(instance.z);
    }
    for seat in &document.seats {
        max_z = max_z.max(seat.z());
    }
    max_z = max_z.max(document.entrance.z());
    for zone in &document.zones {
        max_z = max_z.max(zone.max_z);
    }
    for link in &document.vertical_links {
        max_z = max_z.max(link.lower.z()).max(link.upper.z());
    }
    max_z
}

fn cursor_grid(editor: &SceneEditorState) -> Vec2 {
    Vec2::new(editor.cursor.x as f32, editor.cursor.y as f32)
}

fn cursor_nav_point(editor: &SceneEditorState) -> NavPointFile {
    NavPointFile::at(cursor_grid(editor), editor.cursor_z)
}

fn mark_document_changed(
    editor: &mut SceneEditorState,
    reload_requests: &mut EventWriter<WorldReloadRequested>,
    reason: &'static str,
) {
    editor.dirty = true;
    reload_requests.send(WorldReloadRequested::current_document(reason));
}

fn draw_prefab_handles(
    gizmos: &mut Gizmos,
    editor: &SceneEditorState,
    document: &WorldDocument,
    objects: &Query<(&WorldObjectEntity, &GridPosition)>,
) {
    for (object, grid_position) in objects {
        let selected = editor
            .selected_instance_id
            .as_ref()
            .is_some_and(|selected_id| selected_id == &object.instance_id);
        let color = if selected {
            Color::srgba(0.25, 0.75, 1.0, 0.95)
        } else {
            Color::srgba(0.25, 0.75, 1.0, 0.35)
        };
        let center = grid_to_screen_at(grid_position.value, grid_position.z);
        gizmos.circle_2d(center, if selected { 10.0 } else { 5.0 }, color);
    }

    if let Some(SceneSelection::Prefab(selected_id)) = &editor.selected {
        if !objects
            .iter()
            .any(|(object, _)| &object.instance_id == selected_id)
        {
            if let Some(instance) = document
                .prefab_instances
                .iter()
                .find(|instance| &instance.id == selected_id)
            {
                if let Ok(grid) = instance.grid_vec() {
                    let center = grid_to_screen_at(grid, instance.z);
                    gizmos.circle_2d(center, 10.0, Color::srgba(0.25, 0.75, 1.0, 0.95));
                }
            }
        }
    }
}

fn draw_edge_wall_handles(
    gizmos: &mut Gizmos,
    editor: &SceneEditorState,
    document: &WorldDocument,
) {
    for (index, wall) in document.edge_walls.iter().enumerate() {
        if wall.z != editor.cursor_z {
            continue;
        }
        let Some(cell) = ivec2_from_slice(&wall.grid) else {
            continue;
        };
        let selected = matches!(editor.selected, Some(SceneSelection::EdgeWall(selected)) if selected == index);
        draw_cell_edge(
            gizmos,
            cell,
            wall.z,
            wall.direction,
            if selected {
                Color::srgba(1.0, 0.33, 0.2, 0.95)
            } else {
                Color::srgba(1.0, 0.33, 0.2, 0.45)
            },
        );
    }
}

fn draw_marker_handles(gizmos: &mut Gizmos, editor: &SceneEditorState, document: &WorldDocument) {
    for (index, seat) in document.seats.iter().enumerate() {
        if seat.z() != editor.cursor_z {
            continue;
        }
        if let Ok(grid) = seat.grid_vec() {
            let selected = matches!(editor.selected, Some(SceneSelection::Seat(selected)) if selected == index);
            gizmos.circle_2d(
                grid_to_screen_at(grid, seat.z()),
                if selected { 9.0 } else { 5.0 },
                if selected {
                    Color::srgba(0.3, 1.0, 0.45, 0.95)
                } else {
                    Color::srgba(0.3, 1.0, 0.45, 0.45)
                },
            );
        }
    }

    if document.entrance.z() == editor.cursor_z {
        if let Ok(grid) = document.entrance.grid_vec() {
            let selected = matches!(editor.selected, Some(SceneSelection::Entrance));
            draw_editor_cell(
                gizmos,
                grid,
                document.entrance.z(),
                if selected {
                    Color::srgba(0.3, 0.9, 1.0, 0.95)
                } else {
                    Color::srgba(0.3, 0.9, 1.0, 0.45)
                },
            );
        }
    }
}

fn draw_vertical_link_handles(
    gizmos: &mut Gizmos,
    editor: &SceneEditorState,
    document: &WorldDocument,
) {
    for (index, link) in document.vertical_links.iter().enumerate() {
        let Ok(lower) = link.lower.grid_vec() else {
            continue;
        };
        let Ok(upper) = link.upper.grid_vec() else {
            continue;
        };
        let lower_center = grid_to_screen_at(lower, link.lower.z());
        let upper_center = grid_to_screen_at(upper, link.upper.z());
        let on_active_floor =
            link.lower.z() == editor.cursor_z || link.upper.z() == editor.cursor_z;
        if on_active_floor {
            gizmos.line_2d(lower_center, upper_center, Color::srgba(0.8, 0.5, 1.0, 0.7));
        }
        for (endpoint, point, grid) in [
            (LinkEndpoint::Lower, &link.lower, lower),
            (LinkEndpoint::Upper, &link.upper, upper),
        ] {
            if point.z() != editor.cursor_z {
                continue;
            }
            let selected = matches!(
                editor.selected,
                Some(SceneSelection::VerticalLinkEndpoint {
                    index: selected_index,
                    endpoint: selected_endpoint,
                }) if selected_index == index && selected_endpoint == endpoint
            );
            gizmos.circle_2d(
                grid_to_screen_at(grid, point.z()),
                if selected { 11.0 } else { 6.0 },
                if selected {
                    Color::srgba(0.8, 0.5, 1.0, 0.95)
                } else {
                    Color::srgba(0.8, 0.5, 1.0, 0.45)
                },
            );
        }
    }
}

fn draw_zone_handles(gizmos: &mut Gizmos, editor: &SceneEditorState, document: &WorldDocument) {
    for (index, zone) in document.zones.iter().enumerate() {
        if editor.cursor_z < zone.min_z || editor.cursor_z > zone.max_z {
            continue;
        }
        let Ok(min) = vec2_from_slice(&zone.min) else {
            continue;
        };
        let Ok(max) = vec2_from_slice(&zone.max) else {
            continue;
        };
        let selected =
            matches!(editor.selected, Some(SceneSelection::Zone(selected)) if selected == index);
        draw_zone_outline(
            gizmos,
            min,
            max,
            editor.cursor_z,
            if selected {
                Color::srgba(1.0, 0.86, 0.24, 0.95)
            } else {
                Color::srgba(1.0, 0.86, 0.24, 0.35)
            },
        );
    }
}

fn draw_collision_handles(
    gizmos: &mut Gizmos,
    editor: &SceneEditorState,
    document: &WorldDocument,
) {
    for collision in &document.collision_cells {
        if collision.z != editor.cursor_z {
            continue;
        }
        let Some(cell) = ivec2_from_slice(&collision.grid) else {
            continue;
        };
        draw_editor_cell(
            gizmos,
            cell.as_vec2(),
            collision.z,
            if collision.blocked {
                Color::srgba(1.0, 0.05, 0.05, 0.55)
            } else {
                Color::srgba(0.05, 1.0, 0.6, 0.55)
            },
        );
    }
}

fn draw_editor_cell(gizmos: &mut Gizmos, grid: Vec2, z: i32, color: Color) {
    let center = grid_to_screen_at(grid, z);
    let top = center + Vec2::new(0.0, TILE_HEIGHT * 0.5);
    let right = center + Vec2::new(TILE_WIDTH * 0.5, 0.0);
    let bottom = center + Vec2::new(0.0, -TILE_HEIGHT * 0.5);
    let left = center + Vec2::new(-TILE_WIDTH * 0.5, 0.0);

    gizmos.line_2d(top, right, color);
    gizmos.line_2d(right, bottom, color);
    gizmos.line_2d(bottom, left, color);
    gizmos.line_2d(left, top, color);
}

fn draw_cell_edge(
    gizmos: &mut Gizmos,
    cell: IVec2,
    z: i32,
    direction: EdgeDirectionFile,
    color: Color,
) {
    let center = grid_to_screen_at(cell.as_vec2(), z);
    let top = center + Vec2::new(0.0, TILE_HEIGHT * 0.5);
    let right = center + Vec2::new(TILE_WIDTH * 0.5, 0.0);
    let bottom = center + Vec2::new(0.0, -TILE_HEIGHT * 0.5);
    let left = center + Vec2::new(-TILE_WIDTH * 0.5, 0.0);
    let (start, end) = match direction {
        EdgeDirectionFile::N => (left, top),
        EdgeDirectionFile::E => (top, right),
        EdgeDirectionFile::S => (right, bottom),
        EdgeDirectionFile::W => (bottom, left),
    };
    gizmos.line_2d(start, end, color);
}

fn draw_zone_outline(gizmos: &mut Gizmos, min: Vec2, max: Vec2, z: i32, color: Color) {
    let top = grid_to_screen_at(Vec2::new(min.x, min.y), z);
    let right = grid_to_screen_at(Vec2::new(max.x, min.y), z);
    let bottom = grid_to_screen_at(Vec2::new(max.x, max.y), z);
    let left = grid_to_screen_at(Vec2::new(min.x, max.y), z);

    gizmos.line_2d(top, right, color);
    gizmos.line_2d(right, bottom, color);
    gizmos.line_2d(bottom, left, color);
    gizmos.line_2d(left, top, color);
}

fn vec2_from_slice(value: &[f32]) -> Result<Vec2, String> {
    if value.len() != 2 {
        return Err("expected two f32 values".to_string());
    }
    Ok(Vec2::new(value[0], value[1]))
}

fn ivec2_from_slice(value: &[i32]) -> Option<IVec2> {
    if value.len() == 2 {
        Some(IVec2::new(value[0], value[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_tile_painting_updates_existing_cell_override() {
        let mut document = WorldDocument::empty_demo();
        paint_tile_cell(&mut document, IVec2::new(3, 4), 0, TileKindFile::Porch);
        paint_tile_cell(
            &mut document,
            IVec2::new(3, 4),
            0,
            TileKindFile::TavernFloor,
        );

        let overrides: Vec<&TileRegionFile> = document
            .tile_regions
            .iter()
            .filter(|region| region.from == vec![3, 4] && region.to == vec![3, 4])
            .collect();
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].kind, TileKindFile::TavernFloor);
    }

    #[test]
    fn editor_can_create_edge_wall_marker_link_and_zone_data() {
        let mut document = WorldDocument::empty_demo();
        document.levels = 2;
        document
            .prop_styles
            .push(crate::world::import::PropStyleDefinitionFile {
                name: "test_wall".to_string(),
                style: crate::world::import::PropStyleFile {
                    size: vec![64.0, 48.0],
                    layer: PropLayerFile::Wall,
                    asset_id: String::new(),
                    asset: String::new(),
                    color: vec![0.2, 0.2, 0.2],
                    anchor: crate::world::import::RenderAnchorFile::WallBaseline,
                    y_offset: None,
                    depth_offset: 0.0,
                    shadow: false,
                    footprint: crate::world::import::FootprintFile::None,
                    cutaway: crate::world::import::CutawayFile::None,
                },
            });

        let wall_index = upsert_edge_wall(
            &mut document,
            IVec2::new(2, 3),
            1,
            EdgeDirectionFile::S,
            "test_wall".to_string(),
        );
        document
            .seats
            .push(NavPointFile::at(Vec2::new(4.0, 5.0), 1));
        let link_index = create_vertical_link(
            &mut document,
            NavPointFile::at(Vec2::new(1.0, 1.0), 0),
            NavPointFile::at(Vec2::new(1.0, 1.0), 1),
            VerticalLinkKindFile::Stairs,
            true,
        )
        .expect("different levels create a valid link");
        let zone_index = create_zone(
            &mut document,
            ZoneDraft {
                cell: IVec2::new(1, 1),
                z: 0,
            },
            ZoneDraft {
                cell: IVec2::new(4, 4),
                z: 1,
            },
        );
        let collision_index = upsert_collision_cell(&mut document, IVec2::new(6, 7), 1, true);

        assert_eq!(wall_index, 0);
        assert_eq!(document.edge_walls[0].grid, vec![2, 3]);
        assert_eq!(document.seats[0].z(), 1);
        assert_eq!(link_index, 0);
        assert_eq!(document.vertical_links[0].upper.z(), 1);
        assert_eq!(zone_index, 0);
        assert_eq!(document.zones[0].max_z, 1);
        assert_eq!(collision_index, 0);
        assert!(document.collision_cells[0].blocked);
    }
}
