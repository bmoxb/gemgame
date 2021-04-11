pub mod animations;

use std::collections::HashMap;

use animations::{boxed_continuous, boxed_static};
use array_macro::array;
use lazy_static::lazy_static;
use macroquad::prelude as quad;
use shared::maps::Tile;

const ROCK_SMASH_FRAMES: [animations::Frame; 7] =
    array![index => animations::Frame { at: (index as u16, 3), time: 0.025 }; 7];

const BLUE_FLOWER_FRAMES: &[animations::Frame] =
    &[animations::Frame { at: (5, 2), time: 2.65 }, animations::Frame { at: (6, 2), time: 0.35 }];

const YELLOW_ORANGE_FLOWER_FRAMES: &[animations::Frame] = &[
    animations::Frame { at: (0, 4), time: 1.75 },
    animations::Frame { at: (1, 4), time: 0.25 },
    animations::Frame { at: (2, 4), time: 0.25 },
    animations::Frame { at: (3, 4), time: 0.25 }
];

const WATER_FRAME_TIME: f64 = 0.15;
const WATER_FRAMES: [animations::Frame; 4] =
    array![index => animations::Frame { at: (4 + index as u16, 7), time: WATER_FRAME_TIME }; 4];

const WATER_GRASS_TOP_FRAMES: [animations::Frame; 4] =
    array![index => animations::Frame { at: (4 + index as u16, 6), time: WATER_FRAME_TIME }; 4];

const WATER_GRASS_CORNER_TOP_LEFT: [animations::Frame; 4] =
    array![index => animations::Frame { at: (4 + index as u16, 4), time: WATER_FRAME_TIME }; 4];

const WATER_GRASS_CORNER_TOP_RIGHT: [animations::Frame; 4] =
    array![index => animations::Frame { at: (4 + index as u16, 5), time: WATER_FRAME_TIME }; 4];

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
        map.insert(Tile::Water, boxed_continuous(&WATER_FRAMES));
        map.insert(Tile::WaterGrassTop, boxed_continuous(&WATER_GRASS_TOP_FRAMES));
        map.insert(Tile::WaterGrassBottom, boxed_static(2, 7));
        map.insert(Tile::WaterGrassLeft, boxed_static(1, 6));
        map.insert(Tile::WaterGrassRight, boxed_static(3, 6));
        map.insert(Tile::WaterGrassTopLeft, boxed_static(1, 5));
        map.insert(Tile::WaterGrassTopRight, boxed_static(3, 5));
        map.insert(Tile::WaterGrassBottomLeft, boxed_static(1, 7));
        map.insert(Tile::WaterGrassBottomRight, boxed_static(3, 7));
        map.insert(Tile::WaterGrassCornerTopLeft, boxed_continuous(&WATER_GRASS_CORNER_TOP_LEFT));
        map.insert(Tile::WaterGrassCornerTopRight, boxed_continuous(&WATER_GRASS_CORNER_TOP_RIGHT));
        map.insert(Tile::WaterGrassCornerBottomLeft, boxed_static(2, 6));
        map.insert(Tile::WaterGrassCornerBottomRight, boxed_static(2, 5));

        map
    };
}

pub fn draw_with_stateless_animation(
    tile: Tile, draw_pos: quad::Vec2, draw_size: f32, single_tile_texture_size: u16, texture: quad::Texture2D,
    chunk_corner: bool
) {
    let animation = STATELESS_TILE_ANIMATIONS.get(&tile).unwrap();
    animation.draw(draw_pos, draw_size, single_tile_texture_size, texture);

    #[cfg(debug_assertions)]
    {
        let (radius_multiplier, colour) = {
            if chunk_corner {
                (0.06, quad::DARKPURPLE)
            }
            else {
                (0.03, quad::RED)
            }
        };
        quad::draw_circle(draw_pos.x, draw_pos.y, draw_size * radius_multiplier, colour);
    }
}

/// Draw a grey square at the specified coordinates. This is to act as a place holder while the necessary data is being
/// fetched from the server.
pub fn draw_pending(draw_pos: quad::Vec2, draw_size: f32) {
    let offset = draw_size * 0.2;
    let reduced_size = draw_size - (offset * 2.0);

    quad::draw_rectangle(draw_pos.x + offset, draw_pos.y + offset, reduced_size, reduced_size, quad::DARKGRAY);
}

pub fn new_rock_smash_animation() -> animations::Once {
    animations::Once::new(&ROCK_SMASH_FRAMES)
}
