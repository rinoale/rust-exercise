use bevy::prelude::*;

use crate::config::{TILE_HEIGHT, TILE_WIDTH};
use crate::universe::UniverseLayer;

pub fn grid_to_iso(gx: f32, gy: f32) -> Vec2 {
    Vec2::new((gx - gy) * TILE_WIDTH * 0.5, (gx + gy) * TILE_HEIGHT * 0.5)
}

pub fn grid_to_screen(grid: Vec2) -> Vec2 {
    let iso = grid_to_iso(grid.x, grid.y);
    Vec2::new(iso.x, -iso.y)
}

pub fn iso_translation(grid: Vec2, y_offset: f32, layer: UniverseLayer, depth_offset: f32) -> Vec3 {
    let screen = grid_to_screen(grid);
    Vec3::new(
        screen.x,
        screen.y + y_offset,
        render_depth(grid, layer, depth_offset),
    )
}

pub fn render_depth(grid: Vec2, layer: UniverseLayer, depth_offset: f32) -> f32 {
    (grid.x + grid.y) * 4.0 + layer.base_depth() + depth_offset
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
}
