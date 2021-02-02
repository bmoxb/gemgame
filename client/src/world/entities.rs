use std::collections::HashMap;

use shared::{
    world::{
        entities::{Direction, Entity},
        TileCoords
    },
    Id
};

pub struct PlayerEntity {
    id: Id,
    contained: Entity,
    /// Request number value to be used for the next `ToServer::MoveMyEntity` message. Incremented after the sending of
    /// each message.
    request_number: u32,
    /// Mapping of request numbers to predicted entity coordinates. 'Unverified' in this case means that a
    /// `ToServer::MoveMyEntity` message has been sent and the player entity's coordinates have been changed locally
    /// but a `FromServer::YourEntityMoved` response message from the server has not yet been received so it is not
    /// know whether the predicted coordinates align with those on the server side.
    unverified_requests: HashMap<u32, TileCoords>
}

impl PlayerEntity {
    pub fn new(id: Id, contained: Entity) -> Self {
        PlayerEntity { id, contained, request_number: 0, unverified_requests: HashMap::new() }
    }

    ///
    pub fn move_in_direction(&mut self, d: Direction) {}
}
