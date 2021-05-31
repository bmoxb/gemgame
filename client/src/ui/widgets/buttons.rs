use macroquad::prelude as quad;
use shared::items::Item;

use crate::{AssetManager, TextureKey};

const BUTTON_TEXTURE_TILE_SIZE: u16 = 32;

const BUTTON_UP_RELATIVE_TEXTURE_COORDS: (u16, u16) = (2, 0);
const BUTTON_HOVER_RELATIVE_TEXTURE_COORDS: (u16, u16) = (2, 1);
const BUTTON_DOWN_RELATIVE_TEXTURE_COORDS: (u16, u16) = (1, 1);

const QUANTITY_BARS_TEXTURE_COORDS: (u16, u16) = (0, 3);

const INTERACT_SIZE_MULTIPLIER: f32 = 0.6;

pub trait Button {
    /// Determines whether the button is being hovered over and/or pressed based on mouse position & whether or not the
    /// left mouse button is down. Returns true once when the button is clicked on.
    fn update(&mut self, size: f32) -> bool;

    /// Draws the button to the screen. Should return the absolute position (first pair of values in returned tuple) and
    /// size (second tuple value) that button was drawn.
    fn draw(&self, assets: &AssetManager, size: f32) -> ((f32, f32), f32);
}

pub struct SimpleButton {
    is_hover: bool,
    is_down: bool,
    x: f32,
    y: f32,
    icon_texture_x: u16,
    icon_texture_y: u16
}

impl SimpleButton {
    pub fn new(x: f32, y: f32, icon_texture_x: u16, icon_texture_y: u16) -> Self {
        SimpleButton { is_hover: false, is_down: false, x, y, icon_texture_x, icon_texture_y }
    }
}

impl Button for SimpleButton {
    fn update(&mut self, size: f32) -> bool {
        let (mouse_x, mouse_y) = quad::mouse_position();

        let draw_size = super::calculate_largest_squre_draw_size(size) * INTERACT_SIZE_MULTIPLIER;
        let (draw_x, draw_y) = super::calculate_draw_position(self.x, self.y, draw_size, draw_size);

        let rect = quad::Rect { x: draw_x, y: draw_y, w: draw_size, h: draw_size };

        let was_down = self.is_down;

        self.is_hover = rect.contains(quad::vec2(mouse_x, mouse_y));
        self.is_down = self.is_hover && quad::is_mouse_button_down(quad::MouseButton::Left);

        !was_down && self.is_down
    }

    fn draw(&self, assets: &AssetManager, size: f32) -> ((f32, f32), f32) {
        // Button sizes are calculated as a fraction of either the screen width or height (whichever is larger).

        let draw_size = super::calculate_largest_squre_draw_size(size);
        let dest_size = Some(quad::vec2(draw_size, draw_size));

        // Positions of the buttons are expressed relative to the screen size with each coordinate being within the -0.5
        // to 0.5 range.

        let (draw_x, draw_y) = super::calculate_draw_position(self.x, self.y, draw_size, draw_size);

        quad::draw_texture_ex(
            assets.texture(TextureKey::Ui),
            draw_x,
            draw_y,
            quad::WHITE,
            quad::DrawTextureParams {
                dest_size,
                source: Some(crate::make_texture_source_rect(
                    BUTTON_TEXTURE_TILE_SIZE,
                    match (self.is_hover, self.is_down) {
                        (true, false) => BUTTON_HOVER_RELATIVE_TEXTURE_COORDS,
                        (_, true) => BUTTON_DOWN_RELATIVE_TEXTURE_COORDS,
                        _ => BUTTON_UP_RELATIVE_TEXTURE_COORDS
                    }
                )),
                ..Default::default()
            }
        );

        quad::draw_texture_ex(
            assets.texture(TextureKey::Ui),
            draw_x,
            draw_y,
            quad::WHITE,
            quad::DrawTextureParams {
                dest_size,
                source: Some(crate::make_texture_source_rect(
                    BUTTON_TEXTURE_TILE_SIZE,
                    (self.icon_texture_x, self.icon_texture_y)
                )),
                ..Default::default()
            }
        );

        ((draw_x, draw_y), draw_size)
    }
}

pub struct QuantityButton {
    button: SimpleButton,
    pub quantity: u32
}

impl QuantityButton {
    pub fn new(x: f32, y: f32, icon_texture_x: u16, icon_texture_y: u16) -> Self {
        QuantityButton { button: SimpleButton::new(x, y, icon_texture_x, icon_texture_y), quantity: 0 }
    }
}

impl Button for QuantityButton {
    fn update(&mut self, size: f32) -> bool {
        self.button.update(size)
    }

    fn draw(&self, assets: &AssetManager, size: f32) -> ((f32, f32), f32) {
        let ((draw_x, draw_y), draw_size) = self.button.draw(assets, size);

        if self.quantity > 0 {
            let bar_texture_offset = std::cmp::min(self.quantity as u16 - 1, 3);
            let (bars_texture_x, bars_texture_y) = QUANTITY_BARS_TEXTURE_COORDS;
            let quarter_texture_tile_size = BUTTON_TEXTURE_TILE_SIZE / 4;

            quad::draw_texture_ex(
                assets.texture(TextureKey::Ui),
                draw_x + (draw_size / 4.0),
                draw_y,
                quad::WHITE,
                quad::DrawTextureParams {
                    dest_size: Some(quad::vec2(draw_size / 4.0, draw_size)),
                    source: Some(quad::Rect {
                        x: ((bars_texture_x * BUTTON_TEXTURE_TILE_SIZE)
                            + (bar_texture_offset * quarter_texture_tile_size)) as f32,
                        y: (bars_texture_y * BUTTON_TEXTURE_TILE_SIZE) as f32,
                        w: quarter_texture_tile_size as f32,
                        h: BUTTON_TEXTURE_TILE_SIZE as f32
                    }),
                    ..Default::default()
                }
            );
        }

        ((draw_x, draw_y), draw_size)
    }
}

pub struct PurchaseButton<T> {
    button: SimpleButton,
    pub purchase_item: T
}

impl<T> PurchaseButton<T> {
    pub fn new(x: f32, y: f32, icon_texture_x: u16, icon_texture_y: u16, purchase_item: T) -> Self {
        PurchaseButton { button: SimpleButton::new(x, y, icon_texture_x, icon_texture_y), purchase_item }
    }
}

impl<T: Item> Button for PurchaseButton<T> {
    fn update(&mut self, size: f32) -> bool {
        self.button.update(size)
    }

    fn draw(&self, assets: &AssetManager, size: f32) -> ((f32, f32), f32) {
        self.button.draw(assets, size)
        // TODO: Draw some sort of indicator of item cost.
    }
}
