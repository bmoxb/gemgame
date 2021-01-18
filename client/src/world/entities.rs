use shared::{world::entities::Entity, Id};

pub struct PlayerEntity {
    id: Id,
    contained: Entity
}

impl PlayerEntity {
    pub fn new(id: Id, contained: Entity) -> Self { PlayerEntity { id, contained } }
}
