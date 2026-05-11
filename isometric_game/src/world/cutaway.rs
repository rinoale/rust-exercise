use bevy::prelude::*;

use crate::actors::Player;
use crate::universe::GridPosition;
use crate::world::blueprint::WorldBlueprint;

#[derive(Component)]
pub struct CutawayWhenPlayerInside {
    pub zone: String,
}

pub fn update_cutaways(
    blueprint: Res<WorldBlueprint>,
    player_query: Query<&GridPosition, With<Player>>,
    mut cutaways: Query<(&CutawayWhenPlayerInside, &mut Visibility)>,
) {
    let Ok(player_position) = player_query.get_single() else {
        return;
    };

    for (cutaway, mut visibility) in &mut cutaways {
        *visibility = if blueprint.zone_contains(&cutaway.zone, player_position.value) {
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
}
