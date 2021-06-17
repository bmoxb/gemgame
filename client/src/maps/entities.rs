use std::collections::HashMap;

use shared::{
    gems::{self, Gem},
    items::{self, Item},
    maps::{
        entities::{Direction, Entity},
        Map, TileCoords
    },
    messages, Id
};

use super::{ClientMap, MapRenderer};
use crate::networking::{self, ConnectionTrait};

/// The entity controlled by this client program.
pub struct MyEntity {
    id: Id,
    contained: Entity,
    /// Request number value to be used for the next [`shared::messages::ToServer::MoveMyEntity`] message. Incremented
    /// after the sending of each message.
    next_request_number: u32,
    /// Mapping of request numbers to predicted entity coordinates. 'Unverified' in this case means that a
    /// [`shared::messages::ToServer::MoveMyEntity`] message has been sent and the player entity's coordinates have
    /// been changed locally but a [`shared::messages::FromServer::YourEntityMoved`] response message from the server
    /// has not yet been received so it is not yet known whether the predicted coordinates align with those on the
    /// server side.
    unverified_movements: HashMap<u32, TileCoords>,
    /// When this value reaches 0 then the required amount of time has passed since the player's last movement before
    /// it can move again.
    movement_time_countdown: f32
}

impl MyEntity {
    pub fn new(contained: Entity, id: Id) -> Self {
        MyEntity {
            id,
            contained,
            next_request_number: 0,
            unverified_movements: HashMap::new(),
            movement_time_countdown: 0.0
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.movement_time_countdown -= delta;
    }

    /// Will attempt to move the player entity in the specified direction but will fail if moving now would exceed the
    /// movement speed limit, or if the destination tile is occupied/blocking, or if unable to contact the server.
    pub fn move_towards_checked(
        &mut self, direction: Direction, map: &mut ClientMap, connection: &mut networking::Connection,
        renderer: &mut MapRenderer
    ) -> networking::Result<()> {
        // Check if required amount of time has paced since last movement (i.e. don't exceed maximum movement speed):
        if self.movement_time_countdown <= 0.0 {
            // Check if the position the player wants to move to is free (i.e. not a blocking tile and no other
            // entities persent at that position):
            let new_pos = direction.apply(self.contained.pos);
            if map.is_position_free(new_pos) {
                log::trace!("Moving player entity in direction {} to {}", direction, new_pos);

                // Determine the amount of time needed to move to the destination time:
                let dest_tile = map.loaded_tile_at(new_pos).unwrap_or_default();
                let movement_time = self.contained.movement_time(dest_tile);

                // Handle tile changes based on entity movement (e.g. rock smashing):
                map.some_entity_moved_to(new_pos, renderer);

                // Update the map renderer:
                renderer.my_entity_moved(new_pos, movement_time, dest_tile.get_entity_movement_frame_changes());

                // Locally modify player entity's coordinates & direction:
                self.contained.pos = new_pos;
                self.contained.direction = direction;

                // Inform server that this client's player entity wants to move in a given direction:
                let msg = messages::ToServer::MoveMyEntity { request_number: self.next_request_number, direction };
                connection.send(&msg)?;

                // Add to collection of movement predictions awaiting confirmation from the server:
                self.unverified_movements.insert(self.next_request_number, self.contained.pos);

                // Prepare for next movement:
                self.next_request_number += 1;
                self.movement_time_countdown = movement_time;
            }
            else {
                log::trace!(
                    "Cannot move player entity in direction {} to {} as that position is not free",
                    direction,
                    new_pos
                );
            }
        }

        Ok(())
    }

    /// This method is called from the main game state whenever a [`shared::messages::FromServer::YouCollectedGems`]
    /// message is received.
    pub fn obtained_gems(&mut self, gem_type: Gem, quantity_increase: u32) {
        self.contained.gem_collection.increase_quantity(gem_type, quantity_increase);
    }

    /// This method is called from the main game state whenever a [`shared::messages::FromSever::YourEntityMoved`]
    /// message is received. It is the role of this method to ensure that previous predictions regarding player
    /// entity position after movement were correct.
    pub fn received_movement_reconciliation(
        &mut self, request_number: u32, position: TileCoords, renderer: &mut MapRenderer
    ) {
        if let Some(predicted_position) = self.unverified_movements.get(&request_number) {
            if *predicted_position != position {
                log::warn!(
                    "Client-side movement prediction #{} position {} differs from server reconciliation of {}",
                    request_number,
                    predicted_position,
                    position
                );

                // Update map renderer:
                renderer.my_entity_position_corrected(position);

                // Correct position:
                self.contained.pos = position;
            }
        }
        else {
            log::warn!(
                "Received movement reconciliation for movement request #{} which could not found",
                request_number
            );
        }

        self.unverified_movements.remove(&request_number);
    }

    /// Attempt to purchase a 'bool item' (an item that a player can either 0 or 1 of). Will send a message to the
    /// server informing it of the purchase provided that the player has the required gems and does not already own
    /// the item.
    pub fn purchase_bool_item(
        &mut self, item: items::BoolItem, connection: &mut networking::Connection
    ) -> networking::Result<bool> {
        let (gem, required_gem_quantity) = item.get_price();

        let will_buy = self.contained.gem_collection.get_quantity(gem) >= required_gem_quantity
            && !self.contained.item_inventory.has(item);

        if will_buy {
            connection.send(&messages::ToServer::PurchaseSingleItem(item))?;

            self.contained.item_inventory.give(item);
            self.contained.gem_collection.decrease_quantity(gem, required_gem_quantity);
        }

        Ok(will_buy)
    }

    pub fn purchase_quantitative_item(
        &mut self, item: items::QuantitativeItem, quantity: u32, connection: &mut networking::Connection
    ) -> networking::Result<bool> {
        let (gem, required_gem_quantity_per_item) = item.get_price();
        let total_required_gem_quantity = required_gem_quantity_per_item * quantity;

        let will_buy = self.contained.gem_collection.get_quantity(gem) >= total_required_gem_quantity;

        if will_buy {
            connection.send(&messages::ToServer::PurchaseItemQuantity { item, quantity })?;

            self.contained.item_inventory.give_quantity(item, quantity);
            self.contained.gem_collection.decrease_quantity(gem, total_required_gem_quantity);
        }

        Ok(will_buy)
    }

    pub fn place_bomb(
        &mut self, map: &mut ClientMap, connection: &mut networking::Connection
    ) -> networking::Result<()> {
        // Ensure player has a bomb in inventory to place:
        if self.contained.item_inventory.has_how_many(items::QuantitativeItem::Bomb) >= 1 {
            self.contained.bombs_placed_count += 1;

            // Place the bomb on the map locally:
            map.set_bomb_at(self.contained.pos, self.id);

            // Inform the server:
            connection.send(&messages::ToServer::PlaceBomb)?;

            // Remove placed bomb from inventory:
            self.contained.item_inventory.take_quantity(items::QuantitativeItem::Bomb, 1);
        }

        Ok(())
    }

    /// Detonate all the bombs placed by the player *within currently loaded chunks.*
    pub fn detonate_bombs(
        &mut self, map: &mut ClientMap, renderer: &mut MapRenderer, connection: &mut networking::Connection
    ) -> networking::Result<()> {
        // Remove bombs from map:
        let detonated_bomb_positions =
            map.take_bombs_placed_by_in_and_around_chunk(self.id, self.contained.pos.as_chunk_coords());

        let length = detonated_bomb_positions.len() as i32;

        if length > 0 {
            // Inform server:
            connection.send(&messages::ToServer::DetonateBombs)?;

            // Update count of how many bombs have been placed by the player:
            self.contained.bombs_placed_count -= length;

            // Have renderer animate the exploding bombs:
            renderer.bombs_detonated(detonated_bomb_positions);
        }

        Ok(())
    }

    pub fn how_many_bombs_placed(&self) -> i32 {
        self.contained.bombs_placed_count
    }

    pub fn get_contained_entity(&self) -> &Entity {
        &self.contained
    }

    pub fn get_pos(&self) -> TileCoords {
        self.contained.pos
    }

    pub fn get_gem_collection(&self) -> &gems::Collection {
        &self.contained.gem_collection
    }

    pub fn get_inventory(&self) -> &items::Inventory {
        &self.contained.item_inventory
    }
}
