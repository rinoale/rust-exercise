mod hud;

use bevy::prelude::*;

use crate::GameSet;

use hud::{setup_ui, update_ui};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, update_ui.in_set(GameSet::Ui));
    }
}
