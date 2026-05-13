use bevy::prelude::*;

use crate::actors::Player;
use crate::universe::GridPosition;
use crate::world::SceneEditorState;
use crate::world::blueprint::{WorldBlueprint, WorldSceneEntity};

#[derive(Component)]
pub struct CutawayWhenPlayerInside {
    pub zone: String,
}

pub fn update_cutaways(
    blueprint: Res<WorldBlueprint>,
    player_query: Query<&GridPosition, With<Player>>,
    editor_state: Option<Res<SceneEditorState>>,
    mut renderables: Query<
        (
            &GridPosition,
            &mut Visibility,
            Option<&CutawayWhenPlayerInside>,
        ),
        With<WorldSceneEntity>,
    >,
) {
    let player_position = player_query.get_single().ok();
    let active_z = editor_state
        .as_deref()
        .filter(|editor| editor.enabled)
        .map(|editor| editor.cursor_z)
        .or_else(|| player_position.map(|position| position.z))
        .unwrap_or(0);

    for (grid_position, mut visibility, cutaway) in &mut renderables {
        let hidden_by_floor = grid_position.z > active_z;
        let hidden_by_cutaway = cutaway.is_some_and(|cutaway| {
            player_position.is_some_and(|player_position| {
                blueprint.zone_contains_at(&cutaway.zone, player_position.value, player_position.z)
            })
        });

        *visibility = if hidden_by_floor || hidden_by_cutaway {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::blueprint::WorldZone;

    #[test]
    fn zone_contains_interior_points() {
        let zone = WorldZone::new("main", Vec2::new(5.0, 4.0), Vec2::new(20.0, 14.0));

        assert!(zone.contains(Vec2::new(12.0, 8.0)));
        assert!(!zone.contains(Vec2::new(21.0, 8.0)));
        assert!(!zone.contains_at(Vec2::new(12.0, 8.0), 1));
    }

    #[test]
    fn cutaway_visibility_follows_player_zone() {
        let mut blueprint = WorldBlueprint::new(10, 10);
        blueprint.add_zone(WorldZone::new(
            "main",
            Vec2::new(2.0, 2.0),
            Vec2::new(6.0, 6.0),
        ));

        let mut app = App::new();
        app.insert_resource(blueprint)
            .add_systems(Update, update_cutaways);

        let player = app
            .world_mut()
            .spawn((Player, GridPosition::new(Vec2::new(3.0, 3.0))))
            .id();
        let cutaway = app
            .world_mut()
            .spawn((
                WorldSceneEntity,
                GridPosition::new(Vec2::new(3.0, 3.0)),
                CutawayWhenPlayerInside {
                    zone: "main".to_string(),
                },
                Visibility::Inherited,
            ))
            .id();

        app.update();
        assert!(matches!(
            app.world().get::<Visibility>(cutaway),
            Some(Visibility::Hidden)
        ));

        app.world_mut()
            .entity_mut(player)
            .get_mut::<GridPosition>()
            .expect("player keeps a grid position")
            .value = Vec2::new(7.0, 7.0);

        app.update();
        assert!(matches!(
            app.world().get::<Visibility>(cutaway),
            Some(Visibility::Inherited)
        ));
    }

    #[test]
    fn floor_visibility_hides_levels_above_active_player_level() {
        let blueprint = WorldBlueprint::with_levels(10, 10, 2);
        let mut app = App::new();
        app.insert_resource(blueprint)
            .add_systems(Update, update_cutaways);

        app.world_mut()
            .spawn((Player, GridPosition::new(Vec2::new(3.0, 3.0))));
        let lower = app
            .world_mut()
            .spawn((
                WorldSceneEntity,
                GridPosition::at(Vec2::new(3.0, 3.0), 0),
                Visibility::Inherited,
            ))
            .id();
        let upper = app
            .world_mut()
            .spawn((
                WorldSceneEntity,
                GridPosition::at(Vec2::new(3.0, 3.0), 1),
                Visibility::Inherited,
            ))
            .id();

        app.update();

        assert!(matches!(
            app.world().get::<Visibility>(lower),
            Some(Visibility::Inherited)
        ));
        assert!(matches!(
            app.world().get::<Visibility>(upper),
            Some(Visibility::Hidden)
        ));
    }
}
