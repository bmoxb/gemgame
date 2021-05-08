use macroquad::prelude as quad;

const BACKGROUND_COLOUR: quad::Color = quad::Color::new(0.5712, 0.5712, 0.5712, 1.0);

pub struct Menu {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

impl Menu {
    pub fn update(&self) {
        // ...
    }

    pub fn draw(&self) {
        let (draw_width, draw_height) = super::calculate_draw_size(self.width, self.height);
        let (draw_x, draw_y) = super::calculate_draw_position(self.x, self.y, draw_width, draw_height);

        quad::draw_rectangle(draw_x, draw_y, draw_width, draw_height, BACKGROUND_COLOUR);
        quad::draw_rectangle_lines(draw_x, draw_y, draw_width, draw_height, 12.0, quad::BLACK);
    }
}
