use std::collections::HashMap;

use shared::{
    gems::Gem,
    maps::{
        entities::{Direction, Entity},
        Map, TileCoords
    },
    messages
};

use super::{ClientMap, MapRenderer};
use crate::networking::{self, ConnectionTrait};

/// The entity controlled by this client program.
pub struct MyEntity {
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
    pub fn new(contained: Entity) -> Self {
        MyEntity {
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

    pub fn get_contained_entity(&self) -> &Entity {
        &self.contained
    }

    pub fn get_pos(&self) -> TileCoords {
        self.contained.pos
    }
}
