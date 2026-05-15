use bevy::prelude::*;

use crate::AppMode;
use crate::world::{
    SceneEditorState, SceneSelectorState, WorldDocument, WorldReloadRequested, WorldRuntimeConfig,
    WorldRuntimeStatus, open_sandbox_scene,
};

#[derive(Component)]
pub(super) struct IntroRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IntroAction {
    Start,
    Edit,
}

pub(super) fn setup_intro_ui(mut commands: Commands) {
    commands
        .spawn((
            IntroRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.01, 0.009, 0.72)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(430.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.028, 0.023, 0.94)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Last Call Tavern"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.94, 0.88, 0.68)),
                ));
                panel.spawn((
                    Text::new("Apocalypse tavern management"),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.68, 0.7, 0.64)),
                ));
                spawn_intro_button(panel, "Start", IntroAction::Start);
                spawn_intro_button(panel, "Edit", IntroAction::Edit);
            });
        });
}

fn spawn_intro_button(parent: &mut ChildBuilder, label: &str, action: IntroAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                height: Val::Px(48.0),
                padding: UiRect::horizontal(Val::Px(18.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.18, 0.16, 0.12, 1.0)),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: 19.0,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.91, 0.78)),
        ));
}

pub(super) fn intro_button_input(
    mut interactions: Query<
        (&Interaction, &IntroAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut mode: ResMut<AppMode>,
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
                match *action {
                    IntroAction::Start => {
                        *mode = AppMode::Play;
                        editor.enabled = false;
                        selector.visible = false;
                        status.last_message = "Started current scene".to_string();
                    }
                    IntroAction::Edit => {
                        *mode = AppMode::Edit;
                        selector.visible = false;
                        open_sandbox_scene(
                            &mut config,
                            &mut editor,
                            &mut document,
                            &mut reload_requests,
                        );
                        status.last_message = editor.message.clone();
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.27, 0.235, 0.17, 1.0));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.18, 0.16, 0.12, 1.0));
            }
        }
    }
}

pub(super) fn update_intro_visibility(
    mode: Res<AppMode>,
    mut roots: Query<&mut Visibility, With<IntroRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if *mode == AppMode::Intro {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
