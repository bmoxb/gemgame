mod asset_management;
mod states;
mod world;
mod items;
mod ui;

use raylib::prelude::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    pretty_env_logger::init();

    let (mut handle, thread) = raylib::init()
        .title("Potion/Alchemy Roguelike")
        .size(1280, 720).resizable()
        .build();

    log::info!("Initialised RayLib and created window");

    let mut assets = AssetManager::new(
        "assets/",
        "textures/",
        "colour palettes/",
        asset_management::Palette {
            background_colour: Color::BLANK,
            foreground_colours: [
                Color::from((68, 68, 68, 255)),     // dark
                Color::from((136, 136, 136, 255)),  // medium dark
                Color::from((187, 187, 187, 255)),  // medium light
                Color::from((255, 255, 255, 255))   // light
            ]
        },
        PaletteKey::Coffee // target colour palette
    );

    log::info!("Prepared the asset manager");

    let mut current_state: Box::<dyn states::State>;
    current_state = Box::new(states::MainMenu::new());
    current_state.begin(&mut assets, &mut handle, &thread);

    log::info!("Created initial state - beginning main loop");

    while !handle.window_should_close() {
        // Update game logic:

        let delta = handle.get_frame_time();
        let potential_state_change = current_state.update(&mut handle, delta);

        // Draw to screen:
        {
            let mut draw = handle.begin_drawing(&thread);

            draw.clear_background(assets.get_target_palette().background_colour);

            current_state.draw(&mut draw, &assets);

            draw_debug_text(&mut draw, Color::BLUE, 22);
        }

        // Handle state transition (if necessary):

        if let Some(next_state) = potential_state_change {
            log::info!("Changing state from '{}' to '{}'", current_state.title(), next_state.title());
            current_state = next_state;
            current_state.begin(&mut assets, &mut handle, &thread);
        }
    }

    log::info!("Exited main loop");
}

pub type AssetManager = asset_management::AssetManager<TextureKey, PaletteKey>;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum TextureKey { Tiles }

impl asset_management::AssetKey for TextureKey {
    fn path(&self) -> &str {
        match self {
            TextureKey::Tiles => "tiles.png"
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PaletteKey { Coffee }

impl asset_management::AssetKey for PaletteKey {
    fn path(&self) -> &str {
        match self {
            PaletteKey::Coffee => "coffee.json"
        }
    }
}

fn draw_debug_text(draw: &mut RaylibDrawHandle, col: Color, size: i32) {
    draw.draw_text(format!("Version: {}", VERSION).as_str(), 0, 0, size, col);
    draw.draw_text(format!("Frames: {}/sec", draw.get_fps()).as_str(), 0, size, size, col);
    draw.draw_text(format!("Delta: {:.2}ms", draw.get_frame_time() * 1000.0).as_str(), 0, size * 2, size, col);
}