mod colours;

use macroquad::prelude as quad;
use shared::maps::{
    entities::{Direction, Entity, FacialExpression, HairStyle},
    TileCoords
};

use super::tile_coords_to_vec2;

/// Handles the rendering of a single entity.
#[derive(Default)]
pub struct Renderer {
    /// The position that the entity is currently being drawn at.
    pub current_pos: quad::Vec2,
    /// Network delays/inconsistencies mean that the client is going to be informed of a remote entity's movements by
    /// the server with a fluctuating delay. This means that the renderer may be informed of movements to animate
    /// earlier than expected and so keeps those future movement at the end of this queue in order to achieve smooth
    /// animations of remote entity movements.
    movement_queue: Vec<Movement>,
    time_since_movement_began: f32,
    walk_frame: WalkCycle
}

#[derive(Default)]
struct Movement {
    destination_pos: quad::Vec2,
    movement: quad::Vec2,
    movement_time: f32
}

impl Renderer {
    pub fn new(coords: TileCoords, tile_draw_size: f32) -> Self {
        Renderer { current_pos: tile_coords_to_vec2(coords, tile_draw_size), ..Default::default() }
    }

    /// Begin animated movement of the entity from the given coordinates to the specified destination coordinates.
    pub fn do_movement(&mut self, to_coords: TileCoords, movement_time: f32, tile_draw_size: f32) {
        // Ensure the timer is reset to 0 if the entity is not already moving:

        if self.movement_queue.is_empty() {
            self.time_since_movement_began = 0.0;
        }

        // Add the movement to the queue:

        let from_pos = self.movement_queue.last().map(|movement| movement.destination_pos).unwrap_or(self.current_pos);
        let destination_pos = tile_coords_to_vec2(to_coords, tile_draw_size);

        self.movement_queue.push(Movement {
            destination_pos,
            movement: (destination_pos - from_pos) / movement_time,
            movement_time
        });
    }

    /// Update draw position and animations.
    pub fn update(&mut self, delta: f32) {
        self.time_since_movement_began += delta;

        if let Some(current_movement) = self.movement_queue.first() {
            // Adjust the position at which the entity is to be drawn:
            self.current_pos += current_movement.movement * delta;

            // Once the duration of a single tile movement has passed, ensure entity is positioned at the destination
            // coordinates exactly, reset the movement timer, move on to the next movement in the queue, and advance the
            // movement/walk cycle animation frame:
            if self.time_since_movement_began >= current_movement.movement_time {
                self.current_pos = current_movement.destination_pos;
                self.time_since_movement_began = 0.0;
                self.movement_queue.remove(0);
                self.walk_frame = self.walk_frame.next();
            }
        }

        // Change to a stationary walk cycle frame if the entity has not moved for more than 50ms:
        if self.movement_queue.is_empty() && self.time_since_movement_began >= 0.05 {
            self.walk_frame = self.walk_frame.stationary();
        }
    }

    /// Draw the lower portion of the entity (the body).
    pub fn draw_lower(&self, entity: &Entity, texture: quad::Texture2D, tile_draw_size: f32, tile_texture_size: u16) {
        self.draw_part(
            texture,
            0.0,
            0.0,
            colours::clothing(entity.clothing_colour),
            body_draw_params(entity, self.walk_frame, tile_draw_size, tile_texture_size)
        );
    }

    /// Draw the upper portion of the entity (head, face, hands, etc.)
    pub fn draw_upper(&self, entity: &Entity, texture: quad::Texture2D, tile_draw_size: f32, tile_texture_size: u16) {
        let skin_colour = colours::skin(entity.skin_colour);
        let hair_colour = colours::hair(entity.hair_colour);

        // Head:
        self.draw_part(
            texture,
            0.0,
            tile_draw_size * 0.25,
            skin_colour,
            head_draw_params(entity, self.walk_frame, tile_draw_size, tile_texture_size)
        );

        // Determine whether an addition y position offset needs to be applied to the entity's hair and facial features
        // based on position in the walk cycle (needed to create the bobbing head effect as an entity moves):
        let head_bob = match self.walk_frame {
            WalkCycle::Left | WalkCycle::Right => -(tile_draw_size * 0.0625),
            _ => 0.0
        };

        // Face:

        let (right_eye, left_eye, mouth) = match entity.direction {
            Direction::Down => (Some(0.0), Some(tile_draw_size * 0.5), true),
            Direction::Left => (None, Some(tile_draw_size * 0.25), false),
            Direction::Right => (Some(tile_draw_size * 0.25), None, false),
            Direction::Up => (None, None, false)
        };

        let eye_y_offset = (tile_draw_size * 0.75) + head_bob;

        // Right eye:
        if let Some(x_offset) = right_eye {
            self.draw_part(
                texture,
                x_offset,
                eye_y_offset,
                hair_colour,
                eye_draw_params(entity, false, tile_draw_size, tile_texture_size)
            );
        }
        // Left eye:
        if let Some(x_offset) = left_eye {
            self.draw_part(
                texture,
                x_offset,
                eye_y_offset,
                hair_colour,
                eye_draw_params(entity, true, tile_draw_size, tile_texture_size)
            );
        }
        // Mouth:
        if mouth {
            quad::draw_texture_ex(
                texture,
                self.current_pos.x + (tile_draw_size * 0.25),
                self.current_pos.y + (tile_draw_size * 0.25) + head_bob,
                skin_colour,
                mouth_draw_params(entity, tile_draw_size, tile_texture_size)
            );
        }

        // Hair:

        self.draw_part(
            texture,
            0.0,
            (tile_draw_size * 0.875) + head_bob,
            hair_colour,
            hair_draw_params(entity, tile_draw_size, tile_texture_size)
        );
    }

    /// Draw a component of the entity (hair, eye, etc.) using the specified drawing parameters.
    fn draw_part(
        &self, texture: quad::Texture2D, x_offset: f32, y_offset: f32, colour: quad::Color,
        params: quad::DrawTextureParams
    ) {
        quad::draw_texture_ex(texture, self.current_pos.x + x_offset, self.current_pos.y + y_offset, colour, params);
    }
}

#[derive(Clone, Copy)]
enum WalkCycle {
    BeforeRight,
    Right,
    BeforeLeft,
    Left
}

impl WalkCycle {
    /// Returns the next element in the cycle.
    fn next(&self) -> WalkCycle {
        match self {
            WalkCycle::BeforeRight => WalkCycle::Right,
            WalkCycle::Right => WalkCycle::BeforeLeft,
            WalkCycle::BeforeLeft => WalkCycle::Left,
            WalkCycle::Left => WalkCycle::BeforeRight
        }
    }

    /// Returns the next stationary element (i.e. the entity not mid-step) in the cycle.
    fn stationary(&self) -> WalkCycle {
        match self {
            WalkCycle::Left => WalkCycle::BeforeRight,
            WalkCycle::Right => WalkCycle::BeforeLeft,
            _ => *self
        }
    }
}

impl Default for WalkCycle {
    fn default() -> Self {
        WalkCycle::BeforeRight
    }
}

/// Returns the texture rectangle of the appropriate entity body animation frame. The Boolean value indicates whether or
/// not the draw should be horizontally flipped or not.
fn body_draw_params(
    entity: &Entity, walk_frame: WalkCycle, tile_draw_size: f32, tile_texture_size: u16
) -> quad::DrawTextureParams {
    let (x_offset, walk_frame_flip) = match walk_frame {
        WalkCycle::BeforeRight | WalkCycle::BeforeLeft => (0, false),
        WalkCycle::Right => (1, false),
        WalkCycle::Left => (1, true)
    };

    let (y_offset, flip) = match entity.direction {
        Direction::Down => (0, walk_frame_flip),
        Direction::Up => (1, !walk_frame_flip),
        Direction::Left => (2, !walk_frame_flip),
        Direction::Right => (2, walk_frame_flip)
    };

    quad::DrawTextureParams {
        dest_size: Some(quad::vec2(tile_draw_size, tile_draw_size)),
        source: Some(quad::Rect {
            x: (x_offset * tile_texture_size) as f32,
            y: (y_offset * tile_texture_size) as f32,
            w: tile_texture_size as f32,
            h: tile_texture_size as f32
        }),
        flip_x: flip,
        flip_y: true,
        ..Default::default()
    }
}

fn head_draw_params(
    entity: &Entity, walk_frame: WalkCycle, tile_draw_size: f32, tile_texture_size: u16
) -> quad::DrawTextureParams {
    let mut params = body_draw_params(entity, walk_frame, tile_draw_size, tile_texture_size);

    if let Some(src) = &mut params.source {
        // The texture rects for entity heads are positioned 2 tiles to the right of the body texture rects:
        src.x += (2 * tile_texture_size) as f32;

        // There are no separate texture rects for the heads of entities facing upwards - the ones for entities facing
        // forward are used but flipped:
        if entity.direction == Direction::Up {
            src.y = 0.0;
            params.flip_x = !params.flip_x;
        }
    }

    params
}

fn hair_draw_params(entity: &Entity, tile_draw_size: f32, tile_texture_size: u16) -> quad::DrawTextureParams {
    let x_offset = match entity.hair_style {
        HairStyle::Quiff => 0,
        HairStyle::Mohawk => 1,
        HairStyle::Fringe => 2
    };

    let (y_offset, flip) = match entity.direction {
        Direction::Down => (0, false),
        Direction::Up => (0, true),
        Direction::Right => (1, false),
        Direction::Left => (1, true)
    };

    quad::DrawTextureParams {
        dest_size: Some(quad::vec2(tile_draw_size, tile_draw_size / 2.0)),
        source: Some(quad::Rect {
            x: (x_offset * tile_texture_size) as f32,
            y: ((6 + y_offset) * (tile_texture_size / 2)) as f32,
            w: tile_texture_size as f32,
            h: (tile_texture_size / 2) as f32
        }),
        flip_x: flip,
        flip_y: true,
        ..Default::default()
    }
}

fn eye_draw_params(
    entity: &Entity, left_eye: bool, tile_draw_size: f32, tile_texture_size: u16
) -> quad::DrawTextureParams {
    let x_offset = match entity.facial_expression {
        FacialExpression::Neutral => 0,
        FacialExpression::Shocked => 1,
        FacialExpression::Skeptical => {
            if left_eye {
                1
            }
            else {
                0
            }
        }
        FacialExpression::Angry => 2
    };

    // TODO: Blinking eyes.

    quad::DrawTextureParams {
        dest_size: Some(quad::vec2(tile_draw_size / 2.0, tile_draw_size / 2.0)),
        source: Some(eye_or_mouth_texture_rect(x_offset, 0, tile_texture_size)),
        flip_x: left_eye,
        flip_y: true,
        ..Default::default()
    }
}

fn mouth_draw_params(entity: &Entity, tile_draw_size: f32, tile_texture_size: u16) -> quad::DrawTextureParams {
    let x_relative = match entity.facial_expression {
        FacialExpression::Shocked => 1,
        _ => 0
    };

    quad::DrawTextureParams {
        dest_size: Some(quad::vec2(tile_draw_size / 2.0, tile_draw_size / 2.0)),
        source: Some(eye_or_mouth_texture_rect(x_relative, 1, tile_texture_size)),
        flip_y: true,
        ..Default::default()
    }
}

fn eye_or_mouth_texture_rect(x_relative: u16, y_relative: u16, tile_texture_size: u16) -> quad::Rect {
    quad::Rect {
        x: ((x_relative + 4) * (tile_texture_size / 2)) as f32,
        y: ((y_relative + 2) * (tile_texture_size / 2)) as f32,
        w: (tile_texture_size / 2) as f32,
        h: (tile_texture_size / 2) as f32
    }
}
