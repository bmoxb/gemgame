use macroquad::prelude as quad;
use shared::world::entities::Entity;

/// Represents an entity (whether player or AI controlled) on the client side. In addition to entity information
/// required by both client and server (see [`Entity`]) this structure includes information regarding the rendering and
/// animating of this entity.
pub struct LocalEntity {
    /// The contained [`Entity`] structure contains entity information not specific to just client or server side
    /// operations.
    contained: Entity,
    /// The position (in camera space) at which the entity will be drawn. This is distinct from the entity's tile/grid
    /// position due to the fact that entities are animated ('slide') between tiles.
    draw_pos: quad::Vec2 // animations, etc.
}

impl LocalEntity {
    pub fn new(contained: Entity) -> Self { LocalEntity { contained, draw_pos: quad::vec2(0.0, 0.0) } }
}
