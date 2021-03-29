use macroquad::prelude as quad;
use shared::maps::Tile;

/// Draw the given tile at the specified coordinates.
pub fn draw(tile: &Tile, draw_pos: quad::Vec2, draw_size: f32, texture_size: u16, texture: quad::Texture2D) {
    let rect = texture_rect(tile, texture_size);
    let tint = colour_tint(tile);

    let params = quad::DrawTextureParams {
        dest_size: Some(quad::vec2(draw_size, draw_size)),
        source: Some(rect),
        flip_y: true,
        ..Default::default()
    };

    quad::draw_texture_ex(texture, draw_pos.x, draw_pos.y, tint, params);
}

/// Draw a grey square at the specified coordinates. This is to act as a place holder while the necessary data is being
/// fetched from the server.
pub fn draw_pending_tile(draw_pos: quad::Vec2, draw_size: f32) {
    let offset = draw_size * 0.2;
    let reduced_size = draw_size - (offset * 2.0);

    quad::draw_rectangle(draw_pos.x + offset, draw_pos.y + offset, reduced_size, reduced_size, quad::DARKGRAY);
}

/// Get the rectangle for the given tile within the context of the full tiles texture.
fn texture_rect(tile: &Tile, texture_size: u16) -> quad::Rect {
    let (relative_x, relative_y) = texture_pos_relative(tile);

    quad::Rect {
        x: (relative_x * texture_size) as f32,
        y: (relative_y * texture_size) as f32,
        w: texture_size as f32,
        h: texture_size as f32
    }
}

/// Get the texture rectangle coordinates for the given tile relative to the size in pixels of each indivdual tile
/// texture.
fn texture_pos_relative(tile: &Tile) -> (u16, u16) {
    match tile {
        Tile::Grass => (0, 0),
        Tile::Rock => (4, 2)
    }
}

/// Get the colour/tint for the given tile.
fn colour_tint(_tile: &Tile) -> quad::Color {
    quad::WHITE
}
