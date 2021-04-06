pub mod animations;

use std::collections::HashMap;

use animations::{boxed_continuous, boxed_static};
use lazy_static::lazy_static;
use macroquad::prelude as quad;
use shared::maps::Tile;

const BLUE_FLOWER_FRAMES: &[animations::Frame] =
    &[animations::Frame { at: (5, 2), time: 2.65 }, animations::Frame { at: (6, 2), time: 0.35 }];

const YELLOW_ORANGE_FLOWER_FRAMES: &[animations::Frame] = &[
    animations::Frame { at: (0, 3), time: 1.75 },
    animations::Frame { at: (1, 3), time: 0.25 },
    animations::Frame { at: (2, 3), time: 0.25 },
    animations::Frame { at: (3, 3), time: 0.25 }
];

const WATER_FRAME_TIME: f64 = 0.15;
const WATER_FRAMES: &[animations::Frame] = &[
    animations::Frame { at: (4, 3), time: WATER_FRAME_TIME },
    animations::Frame { at: (5, 3), time: WATER_FRAME_TIME },
    animations::Frame { at: (6, 3), time: WATER_FRAME_TIME },
    animations::Frame { at: (7, 3), time: WATER_FRAME_TIME }
];

const ROCK_SMASH_FRAME_TIME: f64 = 0.025;
const ROCK_SMASH_FRAMES: &[animations::Frame] = &[
    animations::Frame { at: (0, 4), time: ROCK_SMASH_FRAME_TIME },
    animations::Frame { at: (1, 4), time: ROCK_SMASH_FRAME_TIME },
    animations::Frame { at: (2, 4), time: ROCK_SMASH_FRAME_TIME },
    animations::Frame { at: (3, 4), time: ROCK_SMASH_FRAME_TIME },
    animations::Frame { at: (4, 4), time: ROCK_SMASH_FRAME_TIME },
    animations::Frame { at: (5, 4), time: ROCK_SMASH_FRAME_TIME },
    animations::Frame { at: (6, 4), time: ROCK_SMASH_FRAME_TIME }
];

lazy_static! {
    static ref STATELESS_TILE_ANIMATIONS: HashMap<Tile, Box<dyn animations::Animation + Sync>> = {
        let mut map = HashMap::new();

        map.insert(Tile::Grass, boxed_static(0, 0));
        map.insert(Tile::FlowerPatch, boxed_static(0, 1));
        map.insert(Tile::Stones, boxed_static(0, 2));
        map.insert(Tile::Dirt, boxed_static(2, 1));
        map.insert(Tile::DirtGrassTop, boxed_static(2, 0));
        map.insert(Tile::DirtGrassBottom, boxed_static(2, 2));
        map.insert(Tile::DirtGrassLeft, boxed_static(1, 1));
        map.insert(Tile::DirtGrassRight, boxed_static(3, 1));
        map.insert(Tile::DirtGrassTopLeft, boxed_static(1, 0));
        map.insert(Tile::DirtGrassTopRight, boxed_static(3, 0));
        map.insert(Tile::DirtGrassBottomLeft, boxed_static(1, 2));
        map.insert(Tile::DirtGrassBottomRight, boxed_static(3, 2));
        map.insert(Tile::DirtGrassCornerTopLeft, boxed_static(4, 0));
        map.insert(Tile::DirtGrassCornerTopRight, boxed_static(5, 0));
        map.insert(Tile::DirtGrassCornerBottomLeft, boxed_static(4, 1));
        map.insert(Tile::DirtGrassCornerBottomRight, boxed_static(5, 1));
        map.insert(Tile::Rock, boxed_static(6, 0));
        map.insert(Tile::RockEmerald, boxed_static(7, 0));
        map.insert(Tile::RockRuby, boxed_static(7, 1));
        map.insert(Tile::RockDiamond, boxed_static(7, 2));
        map.insert(Tile::RockSmashed, boxed_static(6, 1));
        map.insert(Tile::Shrub, boxed_static(4, 2));
        map.insert(Tile::FlowerBlue, boxed_continuous(BLUE_FLOWER_FRAMES));
        map.insert(Tile::FlowersYellowOrange, boxed_continuous(YELLOW_ORANGE_FLOWER_FRAMES));
        map.insert(Tile::Water, boxed_continuous(WATER_FRAMES));

        map
    };
}

pub fn draw_with_stateless_animation(
    tile: &Tile, draw_pos: quad::Vec2, draw_size: f32, single_tile_texture_size: u16, texture: quad::Texture2D
) {
    let animation = STATELESS_TILE_ANIMATIONS.get(tile).unwrap();
    animation.draw(draw_pos, draw_size, single_tile_texture_size, texture);
}

/// Draw a grey square at the specified coordinates. This is to act as a place holder while the necessary data is being
/// fetched from the server.
pub fn draw_pending(draw_pos: quad::Vec2, draw_size: f32) {
    let offset = draw_size * 0.2;
    let reduced_size = draw_size - (offset * 2.0);

    quad::draw_rectangle(draw_pos.x + offset, draw_pos.y + offset, reduced_size, reduced_size, quad::DARKGRAY);
}

pub fn new_rock_smash_animation() -> animations::Once {
    animations::Once::new(ROCK_SMASH_FRAMES)
}
