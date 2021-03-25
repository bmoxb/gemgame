use macroquad::prelude as quad;
use shared::maps::{
    entities::{Direction, Entity, FacialExpression, HairStyle},
    TileCoords
};

use super::tile_coords_to_vec2;

#[derive(Default)]
pub struct Renderer {
    pub current_pos: quad::Vec2,
    destination_pos: quad::Vec2,
    movement: quad::Vec2,
    current_time: f32,
    movement_time: f32,
    walk_frame: WalkCycle
}

impl Renderer {
    pub fn new(coords: TileCoords, tile_draw_size: f32) -> Self {
        let pos = tile_coords_to_vec2(coords, tile_draw_size);
        Renderer { current_pos: pos, destination_pos: pos, ..Default::default() }
    }

    pub fn do_movement(
        &mut self, from_coords: TileCoords, to_coords: TileCoords, movement_time: f32, tile_draw_size: f32
    ) {
        self.current_pos = tile_coords_to_vec2(from_coords, tile_draw_size);
        self.destination_pos = tile_coords_to_vec2(to_coords, tile_draw_size);
        self.movement = (self.destination_pos - self.current_pos) / movement_time;
        self.current_time = 0.0;
        self.movement_time = movement_time;

        self.walk_frame = self.walk_frame.next();
    }

    /// Update draw position and animations.
    pub fn update(&mut self, delta: f32) {
        self.current_time += delta;
        self.current_pos += self.movement * delta;

        // Once the duration of a single tile movement has passed, ensure entity is positioned at the destination
        // coordinates exactly:
        if self.current_time >= self.movement_time {
            self.current_pos = self.destination_pos;
        }
        // Almost immediately completing a tile movement, reset the walk cycle frame:
        if self.current_time >= self.movement_time * 1.2 {
            self.walk_frame = self.walk_frame.stationary();
        }
    }

    /// Draw the lower portion of the entity (the body).
    pub fn draw_lower(
        &self, entity: &Entity, entities_texture: quad::Texture2D, tile_draw_size: f32, tile_texture_size: u16
    ) {
        quad::draw_texture_ex(
            entities_texture,
            self.current_pos.x,
            self.current_pos.y,
            quad::Color::from_rgba(160, 160, 255, 255),
            body_draw_params(entity, self.walk_frame, tile_draw_size, tile_texture_size)
        );
    }

    /// Draw the upper portion of the entity (head, face, hands, etc.)
    pub fn draw_upper(
        &self, entity: &Entity, entities_texture: quad::Texture2D, tile_draw_size: f32, tile_texture_size: u16
    ) {
        // Head:
        quad::draw_texture_ex(
            entities_texture,
            self.current_pos.x,
            self.current_pos.y + (tile_draw_size * 0.25),
            quad::WHITE,
            head_draw_params(entity, self.walk_frame, tile_draw_size, tile_texture_size)
        );

        // Determine whether an addition y position offset needs to be applied to the entity's hair and facial features
        // based on position in the walk cycle (needed to create the bobbing head effect as an entity moves):
        let y_offset = match self.walk_frame {
            WalkCycle::Left | WalkCycle::Right => -(tile_draw_size * 0.0625),
            _ => 0.0
        };

        // Hair:
        quad::draw_texture_ex(
            entities_texture,
            self.current_pos.x,
            self.current_pos.y + (tile_draw_size * 0.875) + y_offset,
            quad::BROWN,
            hair_draw_params(entity, tile_draw_size, tile_texture_size)
        );

        // Right eye:
        quad::draw_texture_ex(
            entities_texture,
            self.current_pos.x,
            self.current_pos.y + (tile_draw_size * 0.75) + y_offset,
            quad::BROWN,
            eye_draw_params(entity, false, tile_draw_size, tile_texture_size)
        );

        // Left eye:
        quad::draw_texture_ex(
            entities_texture,
            self.current_pos.x + (tile_draw_size * 0.5),
            self.current_pos.y + (tile_draw_size * 0.75) + y_offset,
            quad::BROWN,
            eye_draw_params(entity, true, tile_draw_size, tile_texture_size)
        );

        // Mouth:
        quad::draw_texture_ex(
            entities_texture,
            self.current_pos.x + (tile_draw_size * 0.25),
            self.current_pos.y + (tile_draw_size * 0.25) + y_offset,
            quad::WHITE,
            mouth_draw_params(entity, tile_draw_size, tile_texture_size)
        );
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
    fn next(&self) -> WalkCycle {
        match self {
            WalkCycle::BeforeRight => WalkCycle::Right,
            WalkCycle::Right => WalkCycle::BeforeLeft,
            WalkCycle::BeforeLeft => WalkCycle::Left,
            WalkCycle::Left => WalkCycle::BeforeRight
        }
    }

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
        HairStyle::Fringe => 3
    };

    quad::DrawTextureParams {
        dest_size: Some(quad::vec2(tile_draw_size, tile_draw_size / 2.0)),
        source: Some(quad::Rect {
            x: (x_offset * tile_texture_size) as f32,
            y: (tile_texture_size * 3) as f32,
            w: tile_texture_size as f32,
            h: (tile_texture_size / 2) as f32
        }),
        flip_y: true,
        ..Default::default()
    }
}

fn eye_draw_params(
    entity: &Entity, left_eye: bool, tile_draw_size: f32, tile_texture_size: u16
) -> quad::DrawTextureParams {
    let x_relative = match entity.facial_expression {
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
        source: Some(eye_or_mouth_texture_rect(x_relative, 1.0, tile_texture_size)),
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
        source: Some(eye_or_mouth_texture_rect(x_relative, 1.5, tile_texture_size)),
        flip_y: true,
        ..Default::default()
    }
}

fn eye_or_mouth_texture_rect(x_relative: u16, y_multipler: f32, tile_texture_size: u16) -> quad::Rect {
    quad::Rect {
        x: ((x_relative + 4) * (tile_texture_size / 2)) as f32,
        y: tile_texture_size as f32 * y_multipler,
        w: (tile_texture_size / 2) as f32,
        h: (tile_texture_size / 2) as f32
    }
}
