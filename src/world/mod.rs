mod entities;
mod maps;

use std::{
    time::SystemTime,
    path::{ Path, PathBuf },
    io::Read, fs
};
use yaml_rust::YamlLoader;

struct World {
    title: String,

    directory: PathBuf,

    // Seed for random number generation.
    seed: u32,

    // When this world was created (time since Unix epoch).
    created_epoch: u64
}

impl World {
    fn new(title: String) -> Self {
        let now = time_since_epoch();

        World {
            directory: world_save_directory_path(&title),
            title,
            seed: now as u32,
            created_epoch: now
        }
    }

    fn load(title: String) -> Option<Self> {
        let directory = world_save_directory_path(&title);
        let world_file_path = directory.join("world.yml");

        match fs::File::open(&world_file_path) {
            Ok(mut file) => {
                let mut text = String::new();
                file.read_to_string(&mut text);

                YamlLoader::load_from_str(&text); // TODO

                None
            }

            Err(e) => {
                log::warn!("Failed to load world YAML file '{}' due to error: {:?}", world_file_path.display(), e);
                None // TODO: Should use Result to return nature of error.
            }
        }
    }

    fn save(&self) {}
}

fn time_since_epoch() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(e) => {
            log::error!("Failed to get Unix epoch time: {:?}", e);
            0
        }
    }
}

fn world_save_directory_path(title: &str) -> PathBuf {
    let relative = Path::new("saves/").join(title);
    relative.canonicalize().unwrap_or(relative)
}