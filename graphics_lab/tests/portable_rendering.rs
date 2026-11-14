use graphics_lab::{graphics, portable_cube};

#[test]
fn single_cube_readback_handles_unaligned_rows_rotation_and_invalid_sizes() {
    let mut api = graphics::create(None).unwrap();
    let baseline = api
        .read_pixels(&portable_cube::frame(0.5, 0., 193. / 129.), 193, 129)
        .unwrap();
    assert_eq!(baseline.len(), 193 * 129 * 4);
    assert!(baseline.chunks_exact(4).all(|p| p[3] == 255));
    let foreground = baseline
        .chunks_exact(4)
        .filter(|p| *p != &baseline[..4])
        .count();
    assert!(
        (1000..20000).contains(&foreground),
        "expected a visible cube: {foreground}"
    );
    let rotated = api
        .read_pixels(&portable_cube::frame(1.2, 0.2, 193. / 129.), 193, 129)
        .unwrap();
    assert!(
        baseline
            .iter()
            .zip(&rotated)
            .filter(|(a, b)| a != b)
            .count()
            > 1000
    );
    let resized = api
        .read_pixels(&portable_cube::frame(0.5, 0., 0.5), 64, 128)
        .unwrap();
    assert_eq!(resized.len(), 64 * 128 * 4);
    assert!(
        api.read_pixels(&portable_cube::frame(0., 0., 1.), 0, 100)
            .is_err()
    );
    assert!(
        api.read_pixels(&portable_cube::frame(0., 0., 1.), 4097, 100)
            .is_err()
    );
}

#[test]
fn diagonal_scene_renders_ramps_and_changes_with_camera() {
    let directory = std::env::temp_dir().join(format!("diagonal-portable-{}", std::process::id()));
    let mut game = graphics_lab::diagonal::Game::default();
    game.player.overview();
    let mut renderer = graphics_lab::voxel_renderer::VoxelRenderer::new().unwrap();
    let a = directory.join("overview.png");
    let b = directory.join("walk.png");
    renderer.capture(&game, 321, 241, &a).unwrap();
    game.player.respawn(&game.world);
    renderer.capture(&game, 321, 241, &b).unwrap();
    let decode = |path| {
        let mut reader = png::Decoder::new(std::fs::File::open(path).unwrap())
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut pixels).unwrap();
        pixels
    };
    let first = decode(a);
    let second = decode(b);
    // Exclude HUD bands: scene geometry itself must change.
    let changes = (60..180)
        .flat_map(|y| (60..260).map(move |x| (y * 321 + x) * 4))
        .filter(|&i| first[i..i + 4] != second[i..i + 4])
        .count();
    assert!(
        changes > 3000,
        "camera should change the world image: {changes}"
    );
    std::fs::remove_dir_all(directory).unwrap();
}
