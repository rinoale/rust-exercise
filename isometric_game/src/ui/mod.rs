mod hud;
mod menu;

use bevy::prelude::*;

use crate::GameSet;

use hud::{setup_ui, sync_core_ui_visibility, update_ui};
use menu::{intro_button_input, setup_intro_ui, update_intro_visibility};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_ui, setup_intro_ui).chain())
            .add_systems(
                Update,
                (
                    intro_button_input.in_set(GameSet::Input),
                    update_ui.in_set(GameSet::Ui),
                    sync_core_ui_visibility.in_set(GameSet::Ui),
                    update_intro_visibility.in_set(GameSet::Ui),
                ),
            );
    }
}
