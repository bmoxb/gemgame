//! Module containing all code relating to game 'screens' (e.g. the main menu
//! screen, the settings screen, the gameplay screen, etc.)

trait Screen {
    fn draw(/* ... */) {}
    fn update(delta: f64) /* -> ... */ {}
}

struct MainMenuScreen {}
impl Screen for MainMenuScreen {}

struct GameScreen {
    // game world, entities, etc.
}
impl Screen for GameScreen {}