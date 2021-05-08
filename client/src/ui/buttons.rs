use macroquad::prelude as quad;

use crate::{AssetManager, TextureKey};

const BUTTON_TEXTURE_TILE_SIZE: u16 = 32;

const BUTTON_UP_RELATIVE_TEXTURE_COORDS: (u16, u16) = (0, 0);
const BUTTON_DOWN_RELATIVE_TEXTURE_COORDS: (u16, u16) = (1, 0);

pub struct Button {
    is_down: bool,
    x: f32,
    y: f32,
    icon_texture_x: u16,
    icon_texture_y: u16
}

impl Button {
    pub fn draw(&self, assets: &AssetManager, draw_size: f32) {
        let screen_width = quad::screen_width();
        let screen_height = quad::screen_height();

        // Button sizes are calculated as a fraction of either the screen width or height (whichever is larger). Values
        // are rounded down.

        let absolute_draw_size =
            std::cmp::max((draw_size * screen_width) as usize, (draw_size * screen_height) as usize);
        let dest_size = Some(quad::vec2(absolute_draw_size as f32, absolute_draw_size as f32));

        // Positions of the buttons are expressed relative to the screen size with each coordinate being within the -0.5
        // to 0.5 range.

        let draw_x = (screen_width / 2.0) + (screen_width * self.x) - (absolute_draw_size as f32 / 2.0);
        let draw_y = (screen_height / 2.0) + (screen_height * self.y) - (absolute_draw_size as f32 / 2.0);

        quad::draw_texture_ex(
            assets.texture(TextureKey::Ui),
            draw_x,
            draw_y,
            quad::WHITE,
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
}

pub fn make_purchase_menu_button(x: f32, y: f32) -> Button {
    Button { is_down: false, x, y, icon_texture_x: 0, icon_texture_y: 1 }
}

pub fn make_purchase_button() -> Button {
    unimplemented!()
}
