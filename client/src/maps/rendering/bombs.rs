use array_macro::array;
use lazy_static::lazy_static;
use macroquad::prelude as quad;

use super::animations::{self, Animation};

const UNDETONATED_BOMB_FRAME_TIME: f64 = 0.1;
const UNDETONATED_BOMB_FRAMES: [animations::Frame; 3] =
    array![index => animations::Frame { at: (0, index as u16), time: UNDETONATED_BOMB_FRAME_TIME}; 3];

lazy_static! {
    static ref UNDETONATED_BOMB_ANIMATION: animations::Continuous =
        animations::Continuous::new(&UNDETONATED_BOMB_FRAMES);
}

pub fn draw_undetonated_bomb(draw_pos: quad::Vec2, tile_draw_size: f32, texture: quad::Texture2D) {
    UNDETONATED_BOMB_ANIMATION.draw(draw_pos, tile_draw_size, texture);
}
