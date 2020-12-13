use raylib::prelude::*;

use std::{
    fs,
    collections::HashMap,
    hash::Hash,
    fmt::Debug,
    path::{ Path, PathBuf }
};

use serde::{ Serialize, Deserialize };

pub trait AssetKey: PartialEq + Eq + Hash + Debug + Clone {
    fn path(&self) -> &str;
}

pub struct AssetManager<TextureKey: AssetKey> {
    /// Directory containing texture image files.
    textures_directory: PathBuf,
    loaded_textures: HashMap<TextureKey, Texture2D>,
    /// Directory containing colour palette JSON files.
    palettes_directory: PathBuf,
    pub current_palette: Palette
}

impl<TextureKey: AssetKey> AssetManager<TextureKey> {
    pub fn new(dir: &str, textures_subdir: &str, palettes_subdir: &str) -> Self {
        let directory = Path::new(dir);

        AssetManager {
            textures_directory: directory.join(textures_subdir),
            loaded_textures: HashMap::new(),
            palettes_directory: directory.join(palettes_subdir),
            current_palette: Palette {
                background: Color::GRAY,
                ground: Color::WHITE,
                wall: Color::WHITE,
                ripe_plant: Color::WHITE,
                harvested_plant: Color::WHITE,
                dead_plant: Color::WHITE
            }
        }
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

    pub fn require_texture(&mut self, key: TextureKey, handle: &mut RaylibHandle, thread: &RaylibThread) -> bool {
        if !self.loaded_textures.contains_key(&key) {
            let path = self.textures_directory.join(key.path());

            match handle.load_texture(thread, path.to_str().unwrap()) {
                Ok(texture) => {
                    log::info!("Loaded texture '{:?}' from path: {}", key, path.display());
                    self.loaded_textures.insert(key, texture);
                    return true;
                }

                Err(msg) => log::warn!("Failed to load texture '{:?}' due to error: {}", key, msg)
            }
        }

        false
    }

    pub fn load_palette<Key: AssetKey>(&mut self, key: Key) -> bool {
        let path = self.palettes_directory.join(key.path());

        match fs::File::open(&path) {
            Ok(file) => {
                log::debug!("Opened colour palette JSON file: {}", path.display());

                match serde_json::from_reader::<fs::File, Palette>(file) {
                    Ok(palette) => {
                        log::info!("Loaded colour palette '{:?}' from path: {}", key, path.display());
                        self.current_palette = palette;
                        true
                    }

                    Err(e) => {
                        log::warn!("Failed to parse colour palette '{:?}' data due to error: {}", key, e);
                        false
                    }
                }
            }

            Err(e) => {
                log::warn!("Failed to read colour palette '{:?}' JSON from path '{}' due to error: {}",
                           key, path.display(), e);
                false
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Palette {
    #[serde(with = "ColorDef")] pub background: Color,
    #[serde(with = "ColorDef")] pub ground: Color,
    #[serde(with = "ColorDef")] pub wall: Color,
    #[serde(with = "ColorDef", rename = "ripe plant")] pub ripe_plant: Color,
    #[serde(with = "ColorDef", rename = "harvested plant")] pub harvested_plant: Color,
    #[serde(with = "ColorDef", rename = "dead plant")] pub dead_plant: Color
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
struct ColorDef { r: u8, g: u8, b: u8, a: u8 }