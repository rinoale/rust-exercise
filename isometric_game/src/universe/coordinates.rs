use bevy::prelude::*;

use crate::config::{LEVEL_HEIGHT, TILE_HEIGHT, TILE_WIDTH};
use crate::universe::UniverseLayer;

const LEVEL_DEPTH_STRIDE: f32 = 1000.0;

pub fn grid_to_iso(gx: f32, gy: f32) -> Vec2 {
    Vec2::new((gx - gy) * TILE_WIDTH * 0.5, (gx + gy) * TILE_HEIGHT * 0.5)
}

pub fn grid_to_screen_at(grid: Vec2, z: i32) -> Vec2 {
    let iso = grid_to_iso(grid.x, grid.y);
    Vec2::new(iso.x, -iso.y + z as f32 * LEVEL_HEIGHT)
}

pub fn screen_to_grid_at(screen: Vec2, z: i32) -> Vec2 {
    let adjusted_y = screen.y - z as f32 * LEVEL_HEIGHT;
    let x_minus_y = screen.x / (TILE_WIDTH * 0.5);
    let x_plus_y = -adjusted_y / (TILE_HEIGHT * 0.5);
    Vec2::new((x_minus_y + x_plus_y) * 0.5, (x_plus_y - x_minus_y) * 0.5)
}

pub fn iso_translation_at(
    grid: Vec2,
    z: i32,
    y_offset: f32,
    layer: UniverseLayer,
    depth_offset: f32,
) -> Vec3 {
    let screen = grid_to_screen_at(grid, z);
    Vec3::new(
        screen.x,
        screen.y + y_offset,
        render_depth_at(grid, z, layer, depth_offset),
    )
}

#[cfg(test)]
pub fn render_depth(grid: Vec2, layer: UniverseLayer, depth_offset: f32) -> f32 {
    render_depth_at(grid, 0, layer, depth_offset)
}

pub fn render_depth_at(grid: Vec2, z: i32, layer: UniverseLayer, depth_offset: f32) -> f32 {
    z as f32 * LEVEL_DEPTH_STRIDE + (grid.x + grid.y) * 4.0 + layer.base_depth() + depth_offset
}

pub fn distance_grid(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_isometric_objects_render_above_farther_objects() {
        let farther = render_depth(Vec2::new(10.0, 10.0), UniverseLayer::Character, 0.0);
        let lower = render_depth(Vec2::new(10.0, 11.0), UniverseLayer::Character, 0.0);

        assert!(lower > farther);
    }

    #[test]
    fn nearby_props_and_characters_share_y_sort_band() {
        let table_lower_on_screen = render_depth(Vec2::new(10.0, 11.0), UniverseLayer::Prop, 0.0);
        let character_behind = render_depth(Vec2::new(10.0, 10.0), UniverseLayer::Character, 0.0);
        let character_same_cell =
            render_depth(Vec2::new(10.0, 11.0), UniverseLayer::Character, 0.0);

        assert!(table_lower_on_screen > character_behind);
        assert!(character_same_cell > table_lower_on_screen);
    }

    #[test]
    fn higher_levels_project_up_and_sort_above_lower_levels() {
        let ground = grid_to_screen_at(Vec2::new(10.0, 10.0), 0);
        let upper = grid_to_screen_at(Vec2::new(10.0, 10.0), 1);

        assert!(upper.y > ground.y);
        assert!(
            render_depth_at(Vec2::new(10.0, 10.0), 1, UniverseLayer::Floor, 0.0)
                > render_depth_at(Vec2::new(10.0, 10.0), 0, UniverseLayer::Roof, 0.0)
        );
    }

    #[test]
    fn screen_to_grid_reverses_projection_for_editor_picking() {
        let grid = Vec2::new(7.0, 11.0);
        let screen = grid_to_screen_at(grid, 2);
        let restored = screen_to_grid_at(screen, 2);

        assert!((restored.x - grid.x).abs() < 0.001);
        assert!((restored.y - grid.y).abs() < 0.001);
    }
}
