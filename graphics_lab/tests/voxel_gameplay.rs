use glam::{IVec3, Vec3};
use graphics_lab::voxel::{Block, Game, Input, Player, SIZE, World};

fn flat_world() -> World {
    let mut world = World::empty();
    for x in 0..SIZE.x {
        for z in 0..SIZE.z {
            world.set(IVec3::new(x, 0, z), Some(Block::Stone));
        }
    }
    world
}
fn player_at(feet: Vec3) -> Player {
    Player {
        feet,
        yaw: 0.0,
        pitch: 0.0,
        ..Player::default()
    }
}
fn simulate(player: &mut Player, world: &World, input: Input, seconds: f32, dt: f32) {
    for _ in 0..(seconds / dt).round() as u32 {
        player.tick(world, input, dt);
    }
}

#[test]
fn ray_picking_returns_exact_face_distance_and_respects_reach() {
    let mut world = World::empty();
    let cell = IVec3::new(5, 5, 5);
    world.set(cell, Some(Block::Wood));
    for normal in [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ] {
        let origin = cell.as_vec3() + Vec3::splat(0.5) + normal.as_vec3() * 3.0;
        let hit = world.raycast(origin, -normal.as_vec3(), 2.5).unwrap();
        assert_eq!(hit.cell, cell);
        assert_eq!(hit.normal, normal);
        assert!((hit.distance - 2.5).abs() < 1e-5);
        assert!(world.raycast(origin, -normal.as_vec3(), 2.49).is_none());
    }
    assert!(world.raycast(Vec3::ZERO, Vec3::ZERO, 6.0).is_none());
    assert!(world.raycast(Vec3::ZERO, Vec3::X, 6.0).is_none());
}

#[test]
fn mesh_omits_shared_faces_and_has_outward_winding() {
    let mut world = World::empty();
    world.set(IVec3::ONE, Some(Block::Dirt));
    assert_eq!(world.mesh().len(), 36);
    world.set(IVec3::new(2, 1, 1), Some(Block::Dirt));
    let mesh = world.mesh();
    assert_eq!(mesh.len(), 60);
    for triangle in mesh.chunks_exact(3) {
        let a = Vec3::from_slice(&triangle[0].position);
        let b = Vec3::from_slice(&triangle[1].position);
        let c = Vec3::from_slice(&triangle[2].position);
        assert!(
            (b - a)
                .cross(c - a)
                .dot(Vec3::from_slice(&triangle[0].normal))
                > 0.0
        );
    }
}

#[test]
fn digging_collects_item_and_placement_consumes_selected_item() {
    let mut game = Game {
        world: flat_world(),
        player: player_at(Vec3::new(5.5, 1.0, 8.5)),
        inventory: [0; 4],
        selected: Block::Wood as usize,
    };
    let target = IVec3::new(5, 2, 5);
    game.world.set(target, Some(Block::Wood));
    game.world.set(target - IVec3::Z, Some(Block::Stone));
    assert!(game.dig());
    assert_eq!(game.world.get(target), None);
    assert_eq!(game.inventory[Block::Wood as usize], 1);
    assert!(game.place());
    assert_eq!(game.world.get(target), Some(Block::Wood));
    assert_eq!(game.inventory[Block::Wood as usize], 0);
    let revision = game.world.revision;
    assert!(!game.place());
    assert_eq!(revision, game.world.revision);
}

#[test]
fn placement_rejects_player_overlap_and_outside_world_without_spending() {
    let mut game = Game {
        world: World::empty(),
        player: player_at(Vec3::new(5.5, 1.0, 6.5)),
        ..Game::default()
    };
    game.world.set(IVec3::new(5, 2, 5), Some(Block::Stone));
    let inventory = game.inventory;
    assert!(!game.place());
    assert_eq!(game.inventory, inventory);
    assert_eq!(game.world.get(IVec3::new(5, 2, 6)), None);
    assert!(!game.world.set(IVec3::new(-1, 0, 0), Some(Block::Dirt)));
    assert!(!game.world.set(SIZE, Some(Block::Dirt)));
}

#[test]
fn gravity_jump_landing_and_ceiling_collision() {
    let mut world = flat_world();
    let mut player = player_at(Vec3::new(10.5, 4.0, 10.5));
    simulate(&mut player, &world, Input::default(), 1.0, 1.0 / 60.0);
    assert!(player.grounded);
    assert!((player.feet.y - 1.0).abs() < 0.001);
    player.jump();
    let velocity = player.velocity_y;
    player.jump();
    assert_eq!(player.velocity_y, velocity, "no double jump");
    simulate(&mut player, &world, Input::default(), 0.3, 1.0 / 60.0);
    assert!(
        player.feet.y > 2.0,
        "must jump high enough to climb one block"
    );
    simulate(&mut player, &world, Input::default(), 1.0, 1.0 / 60.0);
    assert!(player.grounded);
    assert!((player.feet.y - 1.0).abs() < 0.001);
    world.set(IVec3::new(10, 3, 10), Some(Block::Stone));
    player.jump();
    for _ in 0..30 {
        player.tick(&world, Input::default(), 1.0 / 60.0);
        assert!(player.feet.y + 1.8 <= 3.001);
    }
    assert!(player.grounded);
}

#[test]
fn movement_is_frame_rate_independent_and_diagonals_are_normalized() {
    let world = flat_world();
    let mut straight = player_at(Vec3::new(20.5, 1.0, 20.5));
    let mut diagonal = player_at(straight.feet);
    let mut slow = player_at(straight.feet);
    let start = straight.feet;
    simulate(
        &mut straight,
        &world,
        Input {
            forward: 1.0,
            right: 0.0,
        },
        1.0,
        1.0 / 120.0,
    );
    simulate(
        &mut slow,
        &world,
        Input {
            forward: 1.0,
            right: 0.0,
        },
        1.0,
        0.1,
    );
    simulate(
        &mut diagonal,
        &world,
        Input {
            forward: 1.0,
            right: 1.0,
        },
        1.0,
        1.0 / 60.0,
    );
    assert!((straight.feet - slow.feet).length() < 0.001);
    assert!(((straight.feet - start).length() - (diagonal.feet - start).length()).abs() < 0.001);
}

#[test]
fn walls_stop_player_at_low_frame_rate_and_allow_sliding() {
    let mut world = flat_world();
    for x in 0..SIZE.x {
        for y in 1..5 {
            world.set(IVec3::new(x, y, 8), Some(Block::Stone));
        }
    }
    let mut player = player_at(Vec3::new(10.5, 1.0, 10.5));
    simulate(
        &mut player,
        &world,
        Input {
            forward: 1.0,
            right: 1.0,
        },
        1.0,
        0.1,
    );
    assert!(player.feet.z >= 9.299 && player.feet.z < 9.31);
    assert!(player.feet.x > 13.0, "should slide along the wall");
}

#[test]
fn digging_underfoot_causes_falling_and_void_fall_respawns_safely() {
    let mut world = flat_world();
    let mut player = player_at(Vec3::new(10.5, 1.0, 10.5));
    player.tick(&world, Input::default(), 0.1);
    world.set(IVec3::new(10, 0, 10), None);
    simulate(&mut player, &world, Input::default(), 0.2, 1.0 / 60.0);
    assert!(player.feet.y < 1.0 && !player.grounded);
    simulate(&mut player, &world, Input::default(), 2.0, 1.0 / 60.0);
    assert!(player.feet.y >= 0.999);
    assert!(
        world
            .get((player.feet - Vec3::Y * 0.01).floor().as_ivec3())
            .is_some()
    );
}
