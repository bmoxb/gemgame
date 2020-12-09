use raylib::prelude::*;

use std::{ collections::HashMap, hash::Hash, fmt::Debug, path::{ Path, PathBuf } };

pub trait AssetKey: PartialEq + Eq + Hash + Debug {
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
        AssetManager {
            textures_directory: directory.join(textures_subdir),
            loaded_textures: HashMap::new(),
            input_palette,
            palettes_directory: directory.join(palettes_subdir),
            loaded_palettes: HashMap::new(),
            target_palette_key
        }
    }

    pub fn texture(&self, key: &TextureKey) -> &Texture2D {
        match self.loaded_textures.get(&key) {
            Some(texture) => texture,
            None => {
                log::error!("Could not get texture by key: {:?}", key);
                panic!()
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
                panic!()
            }
        }
    }

    pub fn require_palette(&mut self, key: &PaletteKey) {
        // TODO: Load colour palette from JSON file...
        unimplemented!()
    }
}

pub struct Palette {
    pub background_colour: Color,
    pub foreground_colours: [Color; 4]
}