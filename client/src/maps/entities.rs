use std::collections::HashMap;

use shared::{
    maps::{
        entities::{Direction, Entity},
        Map, Tile, TileCoords
    },
    messages
};

use super::ClientMap;
use crate::{
    networking::{self, ConnectionTrait},
    rendering
};

/// The entity controlled by this client program.
pub struct MyEntity {
    pub contained: Entity,
    /// Request number value to be used for the next [`shared::messages::ToServer::MoveMyEntity`] message. Incremented
    /// after the sending of each message.
    next_request_number: u32,
    /// Mapping of request numbers to predicted entity coordinates. 'Unverified' in this case means that a
    /// [`shared::messages::ToServer::MoveMyEntity`] message has been sent and the player entity's coordinates have
    /// been changed locally but a [`shared::messages::FromServer::YourEntityMoved`] response message from the server
    /// has not yet been received so it is not yet known whether the predicted coordinates align with those on the
    /// server side.
    unverified_movements: HashMap<u32, TileCoords>,
    /// Time that has passed since the last movement between tiles.
    time_since_last_movement: f32
}

impl MyEntity {
    pub fn new(contained: Entity) -> Self {
        MyEntity {
            time_since_last_movement: contained.movement_time(Tile::default()),
            contained,
            next_request_number: 0,
            unverified_movements: HashMap::new()
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.time_since_last_movement += delta;
    }

    /// Will attempt to move the player entity in the specified direction but will fail if moving now would exceed the
    /// movement speed limit, or if the destination tile is occupied/blocking, or if unable to contact the server.
    pub fn move_towards_checked(
        &mut self, direction: Direction, map: &mut ClientMap, connection: &mut networking::Connection,
        renderer: &mut rendering::maps::Renderer
    ) -> networking::Result<()> {
        // Determine the new position and the amount of time needed to move to it:
        let new_pos = direction.apply(self.contained.pos);
        let dest_tile = map.loaded_tile_at(new_pos).unwrap_or_default();
        let movement_time = self.contained.movement_time(dest_tile);

        // Check if required amount of time has paced since last movement (i.e. don't exceed maximum movement speed:
        if self.time_since_last_movement >= movement_time {
            // Check if the position the player wants to move to is free (i.e. not a blocking tile and no other
            // entities persent at that position):
            if map.is_position_free(new_pos) {
                log::trace!("Moving player entity in direction {} to {}", direction, new_pos);

                // Handle tile changes based on entity movement (e.g. rock smashing):
                map.some_entity_moved_to(new_pos, renderer);

                // Update the map renderer:
                renderer.my_entity_moved(new_pos, movement_time);

                // Locally modify player entity's coordinates & direction:
                self.contained.pos = new_pos;
                self.contained.direction = direction;

                // Inform server that this client's player entity wants to move in a given direction:
                let msg = messages::ToServer::MoveMyEntity { request_number: self.next_request_number, direction };
                connection.send(&msg)?;

                // Add to collection of movement predictions awaiting confirmation from the server:
                self.unverified_movements.insert(self.next_request_number, self.contained.pos);

                self.next_request_number += 1;
                self.time_since_last_movement = 0.0;
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

    /// This method is called from the main game state whenever a [`shared::messages::FromSever::YourEntityMoved`]
    /// message is received. It is the role of this method to ensure that previous predictions regarding player
    /// entity position after movement were correct.
    pub fn received_movement_reconciliation(
        &mut self, request_number: u32, position: TileCoords, renderer: &mut rendering::maps::Renderer
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
}
