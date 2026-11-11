#![cfg(all(target_os = "macos", feature = "metal-app"))]
use glam::{IVec3, Vec3};
use graphics_lab::{
    voxel::{Block, Game, Player, World},
    voxel_renderer::VoxelRenderer,
};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

fn pixels(path: &Path) -> Vec<u8> {
    let mut reader = png::Decoder::new(fs::File::open(path).unwrap())
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    pixels.truncate(frame.buffer_size());
    pixels
}
#[test]
fn real_gpu_updates_world_mesh_after_digging_and_placing() {
    objc::rc::autoreleasepool(|| {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("voxel-render-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let mut game = Game {
            world: World::empty(),
            player: Player {
                feet: Vec3::new(5.5, 1.0, 8.5),
                yaw: 0.0,
                pitch: 0.0,
                ..Player::default()
            },
            inventory: [0; 4],
            selected: Block::Wood as usize,
        };
        game.world.set(IVec3::new(5, 2, 5), Some(Block::Wood));
        game.world.set(IVec3::new(5, 2, 4), Some(Block::Stone));
        let mut renderer = VoxelRenderer::new().unwrap();
        let baseline_path = directory.join("baseline.png");
        renderer.capture(&game, 640, 400, &baseline_path).unwrap();
        let baseline = pixels(&baseline_path);
        assert!(game.dig());
        let dug_path = directory.join("dug.png");
        renderer.capture(&game, 640, 400, &dug_path).unwrap();
        let dug = pixels(&dug_path);
        // Central world region excludes the hotbar and text: geometry must change.
        let changed = (100..300)
            .flat_map(|y| (220..420).map(move |x| (y * 640 + x) * 4))
            .filter(|&i| baseline[i..i + 4] != dug[i..i + 4])
            .count();
        assert!(
            changed > 1000,
            "digging should visibly remove the wood block, changed {changed} pixels"
        );
        assert!(game.place());
        let restored_path = directory.join("restored.png");
        renderer.capture(&game, 640, 400, &restored_path).unwrap();
        assert_eq!(
            baseline,
            pixels(&restored_path),
            "placing the collected wood should restore the frame and item count"
        );
        fs::remove_dir_all(directory).unwrap();
    });
}
