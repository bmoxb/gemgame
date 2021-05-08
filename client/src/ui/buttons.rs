use macroquad::prelude as quad;

use crate::{AssetManager, TextureKey};

const BUTTON_TEXTURE_TILE_SIZE: u16 = 32;

const BUTTON_UP_RELATIVE_TEXTURE_COORDS: (u16, u16) = (0, 0);
const BUTTON_DOWN_RELATIVE_TEXTURE_COORDS: (u16, u16) = (1, 0);

const INTERACT_SIZE_MULTIPLIER: f32 = 0.6;

const NOT_HOVER_COLOUR: quad::Color = quad::Color::new(0.8, 0.8, 0.8, 1.0);
const HOVER_COLOUR: quad::Color = quad::WHITE;

pub fn make_purchase_menu_button(x: f32, y: f32) -> Button {
    Button { is_hover: false, is_down: false, x, y, icon_texture_x: 0, icon_texture_y: 1 }
}

pub fn make_purchase_button() -> Button {
    unimplemented!()
}

pub struct Button {
    is_hover: bool,
    is_down: bool,
    x: f32,
    y: f32,
    icon_texture_x: u16,
    icon_texture_y: u16
}

impl Button {
    pub fn update(&mut self, draw_size: f32, mouse_x: f32, mouse_y: f32) {
        let abs_draw_size = calculate_absolute_draw_size(draw_size) * INTERACT_SIZE_MULTIPLIER;
        let (draw_x, draw_y) = self.calculate_draw_pos_centre_origin(abs_draw_size);

        let rect = quad::Rect { x: draw_x, y: draw_y, w: abs_draw_size, h: abs_draw_size };

        self.is_hover = rect.contains(quad::vec2(mouse_x, mouse_y));
        self.is_down = self.is_hover && quad::is_mouse_button_down(quad::MouseButton::Left);
    }

    pub fn draw(&self, assets: &AssetManager, draw_size: f32) {
        // Button sizes are calculated as a fraction of either the screen width or height (whichever is larger). Values
        // are rounded down.

        let absolute_draw_size = calculate_absolute_draw_size(draw_size);
        let dest_size = Some(quad::vec2(absolute_draw_size, absolute_draw_size));

        // Positions of the buttons are expressed relative to the screen size with each coordinate being within the -0.5
        // to 0.5 range.

        let (draw_x, draw_y) = self.calculate_draw_pos_centre_origin(absolute_draw_size);

        quad::draw_texture_ex(
            assets.texture(TextureKey::Ui),
            draw_x,
            draw_y,
            if self.is_hover { HOVER_COLOUR } else { NOT_HOVER_COLOUR },
            quad::DrawTextureParams {
                dest_size,
                source: Some(crate::make_texture_source_rect(
                    BUTTON_TEXTURE_TILE_SIZE,
                    if self.is_down { BUTTON_DOWN_RELATIVE_TEXTURE_COORDS } else { BUTTON_UP_RELATIVE_TEXTURE_COORDS }
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
    }

    fn calculate_draw_pos(&self) -> (f32, f32) {
        (
            (quad::screen_width() / 2.0) + (quad::screen_width() * self.x),
            (quad::screen_height() / 2.0) + (quad::screen_height() * self.y)
        )
    }

    fn calculate_draw_pos_centre_origin(&self, absolute_draw_size: f32) -> (f32, f32) {
        let (draw_x, draw_y) = self.calculate_draw_pos();
        (draw_x - (absolute_draw_size as f32 / 2.0), draw_y - (absolute_draw_size as f32 / 2.0))
    }
}

fn calculate_absolute_draw_size(draw_size: f32) -> f32 {
    std::cmp::max((draw_size * quad::screen_width()) as usize, (draw_size * quad::screen_height()) as usize) as f32
}
