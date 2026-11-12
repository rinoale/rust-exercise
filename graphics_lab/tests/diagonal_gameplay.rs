use glam::{IVec3, Vec3};
use graphics_lab::{
    diagonal::{Game, Piece, Player, Shape, World},
    voxel::Input,
};

#[test]
fn slopes_dimensions_and_equal_height_stacks_match_the_design() {
    for (shape, angle) in [
        (Shape::Gentle, 22.619865_f32),
        (Shape::Diagonal, 45.),
        (Shape::Steep, 67.380135),
    ] {
        for rotation in 0..4 {
            let piece = Piece::new(IVec3::new(7, 11, -3), shape, rotation);
            let (n, _) = piece
                .planes()
                .into_iter()
                .find(|(n, _)| n.y > 0.01)
                .unwrap();
            assert!((n.y.acos().to_degrees() - angle).abs() < 0.001);
        }
    }
    assert_eq!(
        Shape::Cube.dimensions().y * 12,
        Shape::TallBox.dimensions().y * 5
    );
    let d = Shape::Gentle.dimensions();
    assert_eq!(d.x * d.x + d.y * d.y, 13 * 13);
}

#[test]
fn meshes_are_closed_outward_solids_with_correct_volumes() {
    for shape in Shape::ALL {
        for rotation in 0..4 {
            let piece = Piece::new(IVec3::new(-9, 3, 7), shape, rotation);
            let mut volume = 0.;
            for tri in piece.mesh(0).chunks_exact(3) {
                let p = tri
                    .iter()
                    .map(|v| Vec3::from_slice(&v.position))
                    .collect::<Vec<_>>();
                let cross = (p[1] - p[0]).cross(p[2] - p[0]);
                assert!(cross.length() > 0.01);
                assert!(cross.normalize().dot(Vec3::from_slice(&tri[0].normal)) > 0.999);
                volume += p[0].dot(p[1].cross(p[2])) / 6.;
            }
            let d = piece.size();
            let expected = d.x * d.y * d.z * if shape.is_ramp() { 0.5 } else { 1. };
            assert!(
                (volume - expected).abs() < 0.0001,
                "{shape:?} rotation {rotation}: {volume}"
            );
        }
    }
}

#[test]
fn ray_picking_hits_the_slope_and_passes_through_empty_half() {
    let mut world = World::empty();
    world.add(Piece::new(IVec3::ZERO, Shape::Gentle, 0));
    let hit = world
        .raycast(Vec3::new(1.2, 4., 0.5), -Vec3::Y, 10.)
        .unwrap();
    assert!((hit.point.y - 0.5).abs() < 0.0001);
    assert!(hit.normal.y > 0.9);
    assert!(
        world
            .raycast(Vec3::new(0.2, 0.8, -1.), Vec3::Z, 4.)
            .is_none()
    );
    assert!(
        world
            .raycast(Vec3::new(2.2, 0.8, -1.), Vec3::Z, 4.)
            .is_some()
    );
    assert!(world.raycast(Vec3::ZERO, Vec3::ZERO, 4.).is_none());
}

fn ramp_course(shape: Shape, rotation: u8) -> World {
    let mut world = World::empty();
    for x in -5..6 {
        for z in -5..6 {
            world.add(Piece::new(IVec3::new(x * 5, -5, z * 5), Shape::Cube, 0));
        }
    }
    world.add(Piece::new(IVec3::ZERO, shape, rotation));
    world
}

#[test]
fn all_ramps_are_walkable_in_every_orientation_and_do_not_act_as_boxes() {
    for shape in [Shape::Gentle, Shape::Diagonal, Shape::Steep] {
        for rotation in 0..4 {
            let world = ramp_course(shape, rotation);
            let size = world.pieces.last().unwrap().size();
            let (start, direction) = match rotation {
                0 => (Vec3::new(-0.5, 0.001, 0.5), Vec3::X),
                1 => (Vec3::new(0.5, 0.001, -0.5), Vec3::Z),
                2 => (Vec3::new(size.x + 0.5, 0.001, 0.5), -Vec3::X),
                _ => (Vec3::new(0.5, 0.001, size.z + 0.5), -Vec3::Z),
            };
            let mut player = Player {
                feet: start,
                yaw: direction.x.atan2(-direction.z),
                ..Player::default()
            };
            let mut highest = 0_f32;
            for _ in 0..100 {
                player.tick(
                    &world,
                    Input {
                        forward: 1.,
                        right: 0.,
                    },
                    0.,
                    false,
                    1. / 120.,
                );
                assert!(
                    !world.collides(player.feet),
                    "inside {shape:?} rotation {rotation}: {:?}",
                    player.feet
                );
                highest = highest.max(player.feet.y);
            }
            assert!(
                highest > size.y - 0.03,
                "failed to climb {shape:?} rotation {rotation}: {highest}"
            );
            // Traversing backwards must follow the same surface down to the ground.
            for _ in 0..100 {
                player.tick(
                    &world,
                    Input {
                        forward: -1.,
                        right: 0.,
                    },
                    0.,
                    false,
                    1. / 120.,
                );
            }
            assert!(
                player.feet.y < 0.02,
                "failed descent {shape:?}: {:?}",
                player.feet
            );
        }
    }
}

#[test]
fn walls_block_walking_and_jump_collides_with_a_ceiling() {
    let world = ramp_course(Shape::TallBox, 0);
    let mut player = Player {
        feet: Vec3::new(-0.5, 0.001, 0.5),
        yaw: std::f32::consts::FRAC_PI_2,
        ..Player::default()
    };
    for _ in 0..120 {
        player.tick(
            &world,
            Input {
                forward: 1.,
                right: 0.,
            },
            0.,
            false,
            1. / 120.,
        );
    }
    assert!(player.feet.x <= -0.249);
    assert!(player.feet.y < 0.01);
    let mut world = ramp_course(Shape::Cube, 0);
    world.add(Piece::new(IVec3::new(-5, 10, 0), Shape::Cube, 0));
    player.feet = Vec3::new(-0.5, 0.001, 0.5);
    player.jump();
    let mut highest = 0_f32;
    for _ in 0..120 {
        player.tick(&world, Input::default(), 0., false, 1. / 120.);
        highest = highest.max(player.feet.y);
    }
    assert!(highest > 0.2 && highest < 0.301);
    assert!(player.grounded);
}

#[test]
fn building_snaps_and_rejects_overlap_then_removal_restores_scene() {
    let mut world = World::empty();
    world.add(Piece::new(IVec3::ZERO, Shape::Cube, 0));
    let mut game = Game {
        world,
        player: Player {
            feet: Vec3::new(0.5, -1., 4.),
            yaw: 0.,
            pitch: 0.,
            ..Player::default()
        },
        selected: 0,
        rotation: 0,
        message: String::new(),
    };
    let candidate = game.candidate().unwrap();
    assert_eq!(candidate.origin, IVec3::new(0, 0, 5));
    assert!(game.place());
    assert_eq!(game.world.pieces.len(), 2);
    assert!(!game.can_place(&candidate));
    assert!(game.dig());
    assert_eq!(game.world.pieces.len(), 1);
    assert_eq!(game.world.revision, 3);
    let mut body_piece = Piece::new(IVec3::new(0, -5, 20), Shape::Cube, 0);
    assert!(!game.can_place(&body_piece));
    body_piece.origin = IVec3::new(0, 0, 200);
    assert!(!game.can_place(&body_piece));
}

#[test]
fn demo_contains_each_shape_and_a_safe_start() {
    let game = Game::default();
    for shape in Shape::ALL {
        assert!(game.world.pieces.iter().any(|p| p.shape == shape));
    }
    assert!(!game.world.collides(game.player.feet));
    assert_eq!(game.world.support(game.player.feet, 0.1).unwrap().0, 0.);
    let tower_top = |x| {
        game.world
            .pieces
            .iter()
            .filter(|p| p.origin.x == x && p.origin.z == -35)
            .map(|p| p.origin.y + p.shape.dimensions().y)
            .max()
            .unwrap()
    };
    assert_eq!(tower_top(55), 60);
    assert_eq!(tower_top(65), 60);
}
