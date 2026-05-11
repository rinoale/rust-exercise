use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct IsoRenderable {
    pub y_offset: f32,
    pub layer: UniverseLayer,
    pub depth_offset: f32,
}

impl IsoRenderable {
    pub fn new(y_offset: f32, layer: UniverseLayer) -> Self {
        Self {
            y_offset,
            layer,
            depth_offset: 0.0,
        }
    }

    pub fn with_offset(y_offset: f32, layer: UniverseLayer, depth_offset: f32) -> Self {
        Self {
            y_offset,
            layer,
            depth_offset,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UniverseLayer {
    Ground,
    Floor,
    Decal,
    Prop,
    Wall,
    Character,
}

impl UniverseLayer {
    pub fn base_depth(self) -> f32 {
        match self {
            UniverseLayer::Ground => 0.0,
            UniverseLayer::Floor => 6.0,
            UniverseLayer::Decal => 14.0,
            UniverseLayer::Prop => 42.0,
            UniverseLayer::Wall => 58.0,
            UniverseLayer::Character => 116.0,
        }
    }
}
