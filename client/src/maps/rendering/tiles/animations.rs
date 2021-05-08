use macroquad::prelude as quad;

/// Trait that describes an animation for a map tile.
pub trait Animation {
    fn draw(&self, draw_pos: quad::Vec2, draw_size: f32, single_tile_texture_size: u16, texture: quad::Texture2D) {
        let rect = self.get_texture_rect(single_tile_texture_size);

        let params = quad::DrawTextureParams {
            dest_size: Some(quad::vec2(draw_size, draw_size)),
            source: Some(rect),
            flip_y: true,
            ..Default::default()
        };

        quad::draw_texture_ex(texture, draw_pos.x, draw_pos.y, quad::WHITE, params);
    }

    fn get_texture_rect(&self, single_tile_texture_size: u16) -> quad::Rect {
        let (relative_x, relative_y) = self.get_relative_texture_coords(quad::get_time());

        quad::Rect {
            x: (relative_x * single_tile_texture_size) as f32,
            y: (relative_y * single_tile_texture_size) as f32,
            w: single_tile_texture_size as f32,
            h: single_tile_texture_size as f32
        }
    }

    /// Returns the texture coordinates relative to the size of each tile texture of the current frame.
    fn get_relative_texture_coords(&self, time: f64) -> (u16, u16);
}

/// An animation that is just a single frame (i.e. not animated but implements [`Animation`]). Can be shared between
/// different tiles (see [`super::draw_with_stateless_animation`]).
pub struct Static(pub u16, pub u16);

impl Animation for Static {
    fn get_relative_texture_coords(&self, _time: f64) -> (u16, u16) {
        (self.0, self.1)
    }
}

pub fn boxed_static(x: u16, y: u16) -> Box<dyn Animation + Sync> {
    Box::new(Static(x, y))
}

/// An animation that continuously loops the same set of frames. Can be shared between different tiles (see
/// [`super::draw_with_stateless_animation`]).
pub struct Continuous {
    frames: &'static [Frame],
    duration: f64
}

impl Continuous {
    pub fn new(frames: &'static [Frame]) -> Self {
        Continuous { frames, duration: duration_of_frames(frames) }
    }
}

impl Animation for Continuous {
    fn get_relative_texture_coords(&self, time: f64) -> (u16, u16) {
        frame_at_time(self.frames, time % self.duration).unwrap()
    }
}

pub fn boxed_continuous(frames: &'static [Frame]) -> Box<dyn Animation + Sync> {
    Box::new(Continuous::new(frames))
}

/// An animation that plays only once. This animation type is used for a tile transitions and should not be shared in
/// the same way that continuous and static animations can be.
pub struct Once {
    frames: &'static [Frame],
    start_time: f64,
    duration: f64
}

impl Once {
    pub fn new(frames: &'static [Frame]) -> Self {
        Once { frames, start_time: quad::get_time(), duration: duration_of_frames(frames) }
    }

    pub fn has_concluded(&self) -> bool {
        quad::get_time() >= self.start_time + self.duration
    }
}

impl Animation for Once {
    fn get_relative_texture_coords(&self, time: f64) -> (u16, u16) {
        frame_at_time(self.frames, time - self.start_time).unwrap_or_else(|| self.frames.last().unwrap().at)
    }
}

/// Gets the relative texture coordinates from the given set of frames at the specified time.
fn frame_at_time(frames: &[Frame], frame_time: f64) -> Option<(u16, u16)> {
    let mut current_end_time = 0.0;

    for frame in frames {
        current_end_time += frame.time;

        if frame_time <= current_end_time {
            return Some(frame.at);
        }
    }

    None
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub at: (u16, u16),
    pub time: f64
}

/// Calculates the total duration of a set of animation frames.
fn duration_of_frames(frames: &[Frame]) -> f64 {
    let mut duration = 0.0;

    for frame in frames {
        duration += frame.time;
    }

    duration
}
