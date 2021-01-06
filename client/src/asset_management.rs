use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    path::{Path, PathBuf}
};

use macroquad::prelude as quad;

pub trait AssetKey: PartialEq + Eq + Hash + Debug + Clone + Copy {
    fn path(&self) -> &str;
}

pub struct AssetManager<TextureKey: AssetKey> {
    /// Directory containing texture image files.
    textures_directory: PathBuf,
    loaded_textures: HashMap<TextureKey, quad::Texture2D>
}

impl<TextureKey: AssetKey> AssetManager<TextureKey> {
    pub fn new(dir: &str, textures_subdir: &str) -> Self {
        let directory = Path::new(dir);

        AssetManager { textures_directory: directory.join(textures_subdir), loaded_textures: HashMap::new() }
    }

    pub fn texture(&self, key: TextureKey) -> quad::Texture2D {
        match self.loaded_textures.get(&key) {
            Some(texture) => *texture,
            None => {
                log::error!("Could not get texture by key: {:?}", key);
                panic!()
            }
        }
    }

    pub async fn require_texture(&mut self, key: TextureKey) {
        if !self.loaded_textures.contains_key(&key) {
            let path = self.textures_directory.join(key.path());

            let texture = quad::load_texture(path.to_str().unwrap()).await;
            quad::set_texture_filter(texture, quad::FilterMode::Nearest);

            self.loaded_textures.insert(key, texture);

            log::info!("Loaded texture '{:?}' from path: {}", key, path.display())
        }
    }

    pub async fn required_textures(&mut self, keys: &[TextureKey]) {
        for key in keys {
            self.require_texture(*key).await;
        }
    }
}
