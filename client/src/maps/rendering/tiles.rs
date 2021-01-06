use macroquad::prelude as quad;
use shared::maps::Tile;

/// Draw the given tile at the specified coordinates.
pub fn draw(tile: &Tile, x: f32, y: f32, draw_size: f32, texture_rect_size: f32, texture: quad::Texture2D) {
    let rect = tile_texture_rect(tile, texture_rect_size);
    let tint = tile_colour_tint(tile);

    let params = quad::DrawTextureParams {
        dest_size: Some(quad::vec2(draw_size, draw_size)),
        source: Some(rect),
        rotation: 0.0,
        pivot: None
    };

    quad::draw_texture_ex(texture, x, y, tint, params);
}

/// Draw a grey square at the specified coordinates. This is to act as a place holder while the necessary data is being
/// fetched from the server.
pub fn draw_pending_tile(x: f32, y: f32, draw_size: f32) {
    let offset = draw_size * 0.2;
    let reduced_size = draw_size - offset * 2.0;

    quad::draw_rectangle(x + offset, y + offset, reduced_size, reduced_size, quad::DARKGRAY);
}

fn tile_texture_rect(tile: &Tile, texture_rec_size: f32) -> quad::Rect {
    let (x, y) = tile_texture_pos_relative(tile);

    quad::Rect {
        x: x as f32 * texture_rec_size,
        y: y as f32 * texture_rec_size,
        w: texture_rec_size,
        h: texture_rec_size
    }
}

fn tile_texture_pos_relative(tile: &Tile) -> (u16, u16) {
    match tile {
        Tile::Ground => (0, 0)
    }
}

fn tile_colour_tint(_tile: &Tile) -> quad::Color { quad::WHITE }
