use raylib::prelude::*;

use std::{
    fs,
    collections::HashMap,
    hash::Hash,
    fmt::Debug,
    convert::TryInto,
    path::{ Path, PathBuf }
};

const ERROR_PALETTE: Palette = Palette {
    background_colour: Color::BLACK,
    foreground_colours: [ Color::RED, Color::GREEN, Color::BLUE, Color::WHITE ]
};

pub trait AssetKey: PartialEq + Eq + Hash + Debug + Clone {
    fn path(&self) -> &str;
}

pub struct AssetManager<TextureKey: AssetKey, PaletteKey: AssetKey> {
    /// Directory containing texture image files.
    textures_directory: PathBuf,
    loaded_textures: HashMap<TextureKey, Texture2D>,
    /// The colour palette used in images containing texture data. The colours
    /// in this palette will be replaced by those of the palette specified by
    /// the `target_palette_key` field.
    input_palette: Palette,
    /// Directory containing colour palette JSON files.
    palettes_directory: PathBuf,
    loaded_palettes: HashMap<PaletteKey, Palette>,
    /// Key that indicates the colour palette that loaded textures should be
    /// made to use.
    target_palette_key: PaletteKey
}

impl<TextureKey: AssetKey, PaletteKey: AssetKey> AssetManager<TextureKey, PaletteKey> {
    pub fn new(dir: &str, textures_subdir: &str, palettes_subdir: &str, input_palette: Palette, target_palette_key: PaletteKey) -> Self {
        let directory = Path::new(dir);

        let mut manager = AssetManager {
            textures_directory: directory.join(textures_subdir),
            loaded_textures: HashMap::new(),
            input_palette,
            palettes_directory: directory.join(palettes_subdir),
            loaded_palettes: HashMap::new(),
            target_palette_key: target_palette_key.clone()
        };

        manager.require_palette(target_palette_key);
        manager
    }

    pub fn texture(&self, key: &TextureKey) -> &Texture2D {
        match self.loaded_textures.get(&key) {
            Some(texture) => texture,
            None => {
                log::error!("Could not get texture by key: {:?}", key);
                panic!() // TODO: Return error texture.
            }
        }
    }

    pub fn require_texture(&mut self, key: TextureKey, handle: &mut RaylibHandle, thread: &RaylibThread) {
        if !self.loaded_textures.contains_key(&key) {
            let path = self.textures_directory.join(key.path());

            match Image::load_image(path.to_str().unwrap()) {
                Ok(mut image) => {
                    let zipped = self.input_palette.foreground_colours.iter()
                        .zip(self.palette(&self.target_palette_key).foreground_colours.iter());

                    for (old, new) in zipped {
                        log::debug!("Replacing colour {:?} in image '{}' with colour {:?}", old, path.display(), new);
                        image.color_replace(old, new);
                    }

                    match handle.load_texture_from_image(thread, &image) {
                        Ok(texture) => {
                            log::info!("Loaded texture '{:?}' from path: {}", key, path.display());
                            self.loaded_textures.insert(key, texture);
                        }

                        Err(msg) => log::warn!("Failed to load texture '{:?}' from image due to error: {}", key, msg)
                    }
                }

                Err(msg) => log::warn!("Failure to load image for sake of texture '{:?}' due to error: {}", key, msg)
            }
        }
    }

    pub fn palette(&self, key: &PaletteKey) -> &Palette {
        match self.loaded_palettes.get(&key) {
            Some(palette) => palette,
            None => {
                log::error!("Could not get colour palette by key: {:?}", key);
                &ERROR_PALETTE
            }
        }
    }

    pub fn require_palette(&mut self, key: PaletteKey) {
        if !self.loaded_palettes.contains_key(&key) {
            let path = self.palettes_directory.join(key.path());

            match fs::File::open(&path) {
                Ok(file) => {
                    log::debug!("Opened colour palette JSON file: {}", path.display());

                    match serde_json::from_reader::<fs::File, serde_json::Value>(file) {
                        Ok(json) => {
                            let palette = Palette::from_json(&json, &path);

                            log::info!("Loaded colour palette '{:?}' from path: {}", key, path.display());
                            self.loaded_palettes.insert(key, palette);
                        }

                        Err(e) => {}
                    }
                }

                Err(e) => {}
            }
        }
    }
}

pub struct Palette {
    pub background_colour: Color,
    pub foreground_colours: [Color; 4]
}

impl Palette {
    fn from_json(json: &serde_json::Value, path: &Path) -> Self {
        let background_colour_str = json["background"].as_str().unwrap_or_else(|| {
            log::warn!("Colour palette JSON file '{}' lacks a proper background colour field",
                       path.display());

            "000000"
        });

        let background_colour = Color::from_hex(background_colour_str).unwrap_or_else(|_| {
            log::warn!("Could palette JSON file '{}' has an invalid colour field: {}",
                       path.display(), background_colour_str);

            ERROR_PALETTE.background_colour
        });

        let foreground_colours = match json["foreground"].as_array() {
            Some(vector) => {
                vector.iter().map(|colour_json| {
                    let colour_str = colour_json.as_str().unwrap_or_else(|| {
                        log::warn!("");
                        "000000"
                    });

                    Color::from_hex(colour_str).unwrap_or_else(|_| {
                        log::warn!("");
                        Color::BLACK
                    })
                })
                .collect::<Vec<Color>>().try_into().unwrap_or_else(|_| {
                    log::warn!("");

                    ERROR_PALETTE.foreground_colours
                })
            }

            None => {
                log::warn!("Colour palette JSON file '{}' lacks a proper foreground field",
                           path.display());

                ERROR_PALETTE.foreground_colours
            }
        };

        Palette { background_colour, foreground_colours }
    }
}