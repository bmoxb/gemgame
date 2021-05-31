mod widgets;

use macroquad::prelude as quad;
use shared::{
    items,
    maps::{entities::Entity, ChunkCoords}
};
use widgets::Button;

use crate::{maps::entities::MyEntity, networking, AssetManager};

pub struct Ui {
    large_button_size: f32,
    small_button_size: f32,
    show_purchase_buttons_button: widgets::SimpleButton,
    place_bomb_button: widgets::QuantityButton,
    detonate_bombs_button: widgets::QuantityButton,
    showing_purchase_buttons: bool,
    bool_item_purchase_buttons: Vec<widgets::PurchaseButton<items::BoolItem>>,
    quantitative_item_purchase_buttons: Vec<widgets::PurchaseButton<items::QuantitativeItem>>
}

impl Ui {
    pub fn new(button_size: f32) -> Self {
        Ui {
            large_button_size: button_size,
            small_button_size: button_size * 0.75,
            show_purchase_buttons_button: widgets::SimpleButton::new(-0.4, 0.4, 1, 2),
            place_bomb_button: widgets::QuantityButton::new(0.4, 0.4, 1, 3),
            detonate_bombs_button: widgets::QuantityButton::new(0.3, 0.4, 2, 3),
            showing_purchase_buttons: false,
            bool_item_purchase_buttons: vec![widgets::PurchaseButton::new(
                -0.3,
                0.4,
                3,
                0,
                items::BoolItem::RunningShoes
            )],
            quantitative_item_purchase_buttons: vec![widgets::PurchaseButton::new(
                -0.22,
                0.4,
                3,
                1,
                items::QuantitativeItem::Bomb
            )]
        }
    }

    pub fn update(&mut self, player: &mut MyEntity, connection: &mut networking::Connection) -> networking::Result<()> {
        // Set UI bomb button quantity meter based on how many bombs the player has in their inventory:
        self.place_bomb_button.quantity = player.get_inventory().has_how_many(items::QuantitativeItem::Bomb);

        if self.show_purchase_buttons_button.update(self.large_button_size) {
            // Show item purchase buttons:
            self.showing_purchase_buttons = !self.showing_purchase_buttons;
        }

        if self.place_bomb_button.update(self.large_button_size) {
            // Place bomb:
            // TODO
        }

        if self.detonate_bombs_button.update(self.large_button_size) {
            // Detonated placed bombs:
            // TODO
        }

        if self.showing_purchase_buttons {
            for btn in &mut self.bool_item_purchase_buttons {
                if btn.update(self.small_button_size) {
                    player.purchase_bool_item(btn.purchase_item, connection)?;
                }
            }

            for btn in &mut self.quantitative_item_purchase_buttons {
                if btn.update(self.small_button_size) {
                    // TODO: Quantitative item purchase...
                }
            }
        }

        Ok(())
    }

    pub fn draw(&self, assets: &AssetManager) {
        quad::set_default_camera();

        let large_buttons: &[&dyn Button] =
            &[&self.show_purchase_buttons_button, &self.place_bomb_button, &self.detonate_bombs_button];

        for large_btn in large_buttons {
            large_btn.draw(assets, self.large_button_size);
        }

        if self.showing_purchase_buttons {
            let bool_item_buttons = self.bool_item_purchase_buttons.iter().map(|x| x as &dyn Button);
            let quantitative_item_buttons = self.quantitative_item_purchase_buttons.iter().map(|x| x as &dyn Button);

            for small_btn in bool_item_buttons.chain(quantitative_item_buttons) {
                small_btn.draw(assets, self.small_button_size);
            }
        }
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
