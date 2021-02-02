use std::collections::HashMap;

use shared::{
    messages,
    world::{
        entities::{Direction, Entity},
        maps::TileCoords
    },
    Id
};

use super::maps::ClientMap;
use crate::networking::{self, ConnectionTrait};

pub struct PlayerEntity {
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
    ///
    time_since_last_movement: f32
}

impl PlayerEntity {
    pub fn new(id: Id, contained: Entity) -> Self {
        PlayerEntity {
            id,
            next_request_number: 0,
            unverified_movements: HashMap::new(),
            time_since_last_movement: contained.movement_speed(),
            contained
        }
    }

    pub fn update(&mut self, delta: f32) { self.time_since_last_movement += delta; }

    ///
    pub fn move_in_direction(
        &mut self, direction: Direction, map: &ClientMap, connection: &mut networking::Connection
    ) -> networking::Result<()> {
        // First check if required amount of time has paced since last movement (i.e. don't exceed maximum movement
        // speed:
        if self.time_since_last_movement >= self.contained.movement_speed() {
            // TODO: Check for blocking tiles and entities on map...

            log::trace!("Moving player entity in direction {}", direction);

            // Locally modify player entity's coordinates:
            match direction {
                Direction::Down => self.contained.pos.y -= 1,
                Direction::Up => self.contained.pos.y += 1,
                Direction::Left => self.contained.pos.x -= 1,
                Direction::Right => self.contained.pos.x += 1
            }

            // Inform server that this client's player entity wants to move in a given direction:
            let msg = messages::ToServer::MoveMyEntity { request_number: self.next_request_number, direction };
            connection.send(&msg)?;

            // Add to collection of movement predictions awaiting confirmation from the server:
            self.unverified_movements.insert(self.next_request_number, self.contained.pos);

            self.next_request_number += 1;
            self.time_since_last_movement = 0.0;
        }
        Ok(())
    }

    /// This method is called from the main game state whenever a [`shared::messages::FromSever::YourEntityMoved`]
    /// message is received. It is the role of this method to ensure that previous predictions regarding player
    /// entity position after movement were correct.
    pub fn received_movement_reconciliation(&mut self, request_number: u32, position: TileCoords) {}
}
