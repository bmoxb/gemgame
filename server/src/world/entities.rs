pub use shared::entities::Entity;

/// Represents an entity controlled by a player (i.e. controlled remotedly by a specific client).
pub struct PlayerEntity {
    contained: Entity // inventory, etc.
}

/// Represents an entity controlled by the server.
pub struct NonPlayerEntity {
    contained: Entity,
    controller: Box<dyn Controller>
}

impl NonPlayerEntity {
    fn update(&mut self) { self.controller.update(&mut self.contained); }
}

pub trait Controller {
    fn update(&mut self, e: &mut Entity);
}
