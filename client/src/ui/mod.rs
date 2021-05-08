mod buttons;

use buttons::Button;
use macroquad::prelude as quad;
use shared::maps::{entities::Entity, ChunkCoords};

use crate::AssetManager;

pub struct Ui {
    button_draw_size: f32,
    /// Whether or not the item purchase menu is currently shown.
    purchase_menu_open: bool,
    purchase_menu_button: Button
}

impl Ui {
    pub fn new(button_draw_size: f32) -> Self {
        Ui {
            button_draw_size,
            purchase_menu_open: false,
            purchase_menu_button: buttons::make_purchase_menu_button(-0.4, 0.4)
        }
    }

    pub fn update(&mut self) {
        let (mouse_x, mouse_y) = quad::mouse_position();

        self.purchase_menu_button.update(self.button_draw_size, mouse_x, mouse_y);
    }

    pub fn draw(&self, assets: &AssetManager) {
        quad::set_default_camera();

        self.purchase_menu_button.draw(assets, self.button_draw_size);
    }
}

/// Draws debug information to the screen.
#[cfg(debug_assertions)]
pub fn draw_debug_text(
    font_size: f32, font_colour: quad::Color, assets: &AssetManager, my_entity: &Entity,
    loaded_chunk_coords: impl Iterator<Item = ChunkCoords>
) {
    quad::set_default_camera();

    let mut loaded_chunks_string = String::new();
    for coords in loaded_chunk_coords {
        loaded_chunks_string += &format!("({}, {}) ", coords.x, coords.y);
    }

    let msgs = &[
        format!("Version: {}", shared::VERSION),
        format!("Frames: {}/sec", quad::get_fps()),
        format!("Delta: {:.2}ms", quad::get_frame_time() * 1000.0),
        format!("Textures loaded: {}", assets.count_loaded_textures()),
        format!(
            "Player entity position: {}, {}, {}",
            my_entity.pos,
            my_entity.pos.as_chunk_coords(),
            my_entity.pos.as_chunk_offset_coords()
        ),
        format!("Player entity direction: {:?}", my_entity.direction),
        format!("Loaded chunks: {}", loaded_chunks_string)
    ];

    for (i, msg) in msgs.iter().rev().enumerate() {
        quad::draw_text(&msg, 0.0, quad::screen_height() - ((i as f32 + 1.5) * font_size), font_size, font_colour);
    }
}
