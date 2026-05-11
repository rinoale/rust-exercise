use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum RenderAnchor {
    TileCenter,
    LowProp,
    WallBaseline,
    Foot,
    CustomYOffset(f32),
}

impl RenderAnchor {
    pub fn y_offset(self, size: Vec2) -> f32 {
        match self {
            RenderAnchor::TileCenter => 0.0,
            RenderAnchor::LowProp => size.y * 0.45,
            RenderAnchor::WallBaseline => size.y * 0.5,
            RenderAnchor::Foot => size.y * 0.5,
            RenderAnchor::CustomYOffset(offset) => offset,
        }
    }
}
