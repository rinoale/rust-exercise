use bevy::prelude::*;

use crate::config::{CHARACTER_SPRITE_HEIGHT, CHARACTER_SPRITE_WIDTH, TILE_HEIGHT, TILE_WIDTH};
use crate::rendering::RenderAnchor;

pub const CHARACTER_RENDER_SIZE: Vec2 = Vec2::new(CHARACTER_SPRITE_WIDTH, CHARACTER_SPRITE_HEIGHT);

pub fn tile_sprite(image: Handle<Image>) -> Sprite {
    let mut sprite = Sprite::from_image(image);
    sprite.custom_size = Some(Vec2::new(TILE_WIDTH, TILE_HEIGHT));
    sprite
}

pub fn character_sprite(image: Handle<Image>) -> Sprite {
    let mut sprite = Sprite::from_image(image);
    sprite.custom_size = Some(CHARACTER_RENDER_SIZE);
    sprite
}

pub fn character_y_offset() -> f32 {
    RenderAnchor::Foot.y_offset(CHARACTER_RENDER_SIZE)
}
