use macroquad::prelude as quad;

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

    fn get_relative_texture_coords(&self, time: f64) -> (u16, u16);
}

pub struct Static(pub u16, pub u16);

impl Animation for Static {
    fn get_relative_texture_coords(&self, _time: f64) -> (u16, u16) {
        (self.0, self.1)
    }
}

pub fn boxed_static(x: u16, y: u16) -> Box<dyn Animation + Sync> {
    Box::new(Static(x, y))
}

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

pub struct Once {
    frames: &'static [Frame],
    end_time: f64
}

impl Once {
    pub fn new(frames: &'static [Frame]) -> Self {
        Once { frames, end_time: quad::get_time() + duration_of_frames(frames) }
    }

    pub fn has_concluded(&self) -> bool {
        quad::get_time() >= self.end_time
    }
}

impl Animation for Once {
    fn get_relative_texture_coords(&self, time: f64) -> (u16, u16) {
        frame_at_time(self.frames, time - self.end_time).unwrap_or_else(|| self.frames.last().unwrap().at)
    }
}

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

pub struct Frame {
    pub at: (u16, u16),
    pub time: f64
}

fn duration_of_frames(frames: &[Frame]) -> f64 {
    let mut duration = 0.0;

    for frame in frames {
        duration += frame.time;
    }

    duration
}
