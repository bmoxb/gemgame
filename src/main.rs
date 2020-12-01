mod states;
mod world;
mod items;
mod ui;

use raylib::prelude::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn main() {
    pretty_env_logger::init();

    let (mut handle, thread) = raylib::init()
        .title("Potion/Alchemy Roguelike")
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .build();

    log::info!("Initialised RayLib and created window");

    let mut current_state: Box::<dyn states::State>;
    current_state = Box::new(states::MainMenu::new());

    log::info!("Created initial state - beginning main loop");

    while !handle.window_should_close() {
        // Update game logic:

        let delta = handle.get_frame_time();
        let potential_state_change = current_state.update(&mut handle, delta);

        // Draw to screen:

        let mut draw = handle.begin_drawing(&thread);

        draw.clear_background(Color::BLACK);

        current_state.draw(&mut draw);

        draw_debug_text(&mut draw, Color::BLUE, 22);

        // Handle state transition (if necessary):

        if let Some(next_state) = potential_state_change {
            log::info!("Changing state from '{}' to '{}'", current_state.title(), next_state.title());
            current_state = next_state;
        }
    }

    log::info!("Exited main loop");
}

fn draw_debug_text(draw: &mut RaylibDrawHandle, col: Color, size: i32) {
    draw.draw_text(format!("Version: {}", VERSION).as_str(), 0, 0, size, col);
    draw.draw_text(format!("Frames: {}/sec", draw.get_fps()).as_str(), 0, size, size, col);
    draw.draw_text(format!("Delta: {:.2}ms", draw.get_frame_time() * 1000.0).as_str(), 0, size * 2, size, col);
}