use raylib::prelude::*;

use std::{ collections::HashMap, hash::Hash, fmt::Debug, path::{ Path, PathBuf } };

pub trait AssetKey: PartialEq + Eq + Hash + Debug {
    fn path(&self) -> &str;
}

pub struct AssetManager<TextureKey: AssetKey> {
    textures_directory: PathBuf,
    textures: HashMap<TextureKey, Texture2D>
}

impl<TextureKey: AssetKey> AssetManager<TextureKey> {
    pub fn new(dir: &str, textures_subdir: &str) -> Self {
        AssetManager {
            textures_directory: Path::new(dir).join(textures_subdir),
            textures: HashMap::new()
        }
    }

    pub fn texture(&self, key: &TextureKey) -> &Texture2D {
        self.textures.get(&key).expect("Cannot get reference to unloaded texture!")
    }

    pub fn require_texture(&mut self, key: TextureKey, handle: &mut RaylibHandle, thread: &RaylibThread) {
        if !self.textures.contains_key(&key) {
            let path = self.textures_directory.join(key.path());

            match handle.load_texture(thread, path.to_str().unwrap()) {
                Ok(texture) => {
                    log::info!("Loaded texture '{:?}' from path: {}", key, path.display());
                    self.textures.insert(key, texture);
                }

                Err(msg) => log::warn!("Failed to load texture '{:?}' due to error: {}", key, msg)
            }
        }
    }
}