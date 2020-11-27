mod entities;
mod maps;

use std::{
    time::SystemTime,
    path::{ Path, PathBuf },
    fs
};

const SAVES_DIRECTORY: &'static str = "saves/";
const WORLD_JSON_FILE: &'static str = "world.json";

pub struct World {
    title: String,

    // Directory containing world data.
    directory: PathBuf,

    // Seed for random number generation.
    seed: u32,

    // When this world was created (time since Unix epoch).
    created_epoch: u64
}

impl World {
    pub fn new(title: String) -> Self {
        let now = time_since_epoch();

        World {
            directory: world_save_directory_path(&title),
            title,
            seed: now as u32,
            created_epoch: now
        }
    }

    pub fn load(title: String) -> Option<Self> {
        let directory = world_save_directory_path(&title);
        let world_file_path = directory.join(WORLD_JSON_FILE);


        match fs::File::open(&world_file_path) {
            Ok(file) => {
                log::info!("Opened world JSON file: {}", world_file_path.display());

                match serde_json::from_reader::<fs::File, serde_json::Value>(file) {
                    Ok(json) => {
                        log::debug!("World JSON file data: {:?}", json);

                        if let Some(version) = json["version"].as_str() {
                            if version != crate::VERSION {
                                log::warn!("World '{}' is version '{}' which differs from game version '{}'",
                                           title, version, crate::VERSION);
                            }
                        }
                        else {
                            log::warn!("World '{}' does not have a version specified",
                                       title);
                        }

                        let now = time_since_epoch();

                        let seed = json["seed"].as_u64().unwrap_or(now) as u32;
                        let created_epoch = json["created"].as_u64().unwrap_or(now);

                        log::info!("Loaded world: {}", title);

                        Some(World { title, directory, seed, created_epoch })
                    }

                    Err(e) => {
                        log::warn!("Failed to parse world JSON file '{}' due to JSON error: {:?}",
                                   world_file_path.display(), e);
                        None
                    }
                }
            }

            Err(e) => {
                log::warn!("Failed to load world JSON file '{}' due to IO error: {:?}",
                           world_file_path.display(), e);
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
    let relative = Path::new(SAVES_DIRECTORY).join(title);
    relative.canonicalize().unwrap_or(relative)
}

#[cfg(test)]
mod test {
    // TODO: ...
}