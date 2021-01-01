mod states;
mod networking;
mod maps;

use macroquad::prelude as quad;

#[macroquad::main("Client")]
async fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    pretty_env_logger::init(); // Only have logging when targeting desktop.

    let mut current_state: Box<dyn states::State> = Box::new(states::pregame::ConnectingState::new());

    log::info!("Created initial state '{}' - beginning main loop...", current_state.title());

    loop {
        // Update game logic and draw:

        quad::clear_background(quad::BLACK);

        let delta = quad::get_frame_time();
        let potential_state_change = current_state.update_and_draw(delta);

        draw_debug_text(32.0, quad::RED);

        quad::next_frame().await;

        // Handle state transition (if necessary):

        if let Some(next_state) = potential_state_change {
            log::info!("Changing state from '{}' to '{}'", current_state.title(),
                                                           next_state.title());
            current_state = next_state;
        }
    }
}

fn draw_debug_text(size: f32, col: quad::Color) {
    quad::draw_text(&format!("Frames: {}/sec", quad::get_fps()), 0.0, quad::screen_height() - size, size, col);
    quad::draw_text(&format!("Delta: {:.2}ms", quad::get_frame_time() * 1000.0), 0.0, quad::screen_height() - (size * 2.0), size, col);
}