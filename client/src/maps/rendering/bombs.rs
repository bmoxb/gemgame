use array_macro::array;
use lazy_static::lazy_static;
use macroquad::prelude as quad;

use super::animations::{self, Animation};

const UNDETONATED_BOMB_FRAME_TIME: f64 = 0.1;
const UNDETONATED_BOMB_FRAMES: [animations::Frame; 3] =
    array![index => animations::Frame { at: (12, index as u16), time: UNDETONATED_BOMB_FRAME_TIME }; 3];

lazy_static! {
    static ref UNDETONATED_BOMB_ANIMATION: animations::Continuous =
        animations::Continuous::new(&UNDETONATED_BOMB_FRAMES);
}

pub fn draw_undetonated_bomb(draw_pos: quad::Vec2, tile_draw_size: f32, texture: quad::Texture2D) {
    UNDETONATED_BOMB_ANIMATION.draw(draw_pos, super::SINGLE_TILE_TEXTURE_SIZE, tile_draw_size, texture);
}

const DETONATING_BOMB_FRAME_TIME: f64 = 0.08;
const DETONATING_BOMB_FRAMES: [animations::Frame; 4] =
    array![index => animations::Frame { at: (index as u16, 0), time: DETONATING_BOMB_FRAME_TIME }; 4];

/// A single frame of a detonating bomb animation is 3 times the size of a single tile.
pub const DETONATING_BOMB_FRAME_SIZE_MULTIPLIER: u16 = 3;

pub fn make_detonating_bomb_animation() -> animations::Once {
    animations::Once::new(&DETONATING_BOMB_FRAMES)
}
