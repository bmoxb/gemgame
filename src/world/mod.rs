mod entities;
mod maps;

use std::{
    time::SystemTime,
    path::{ Path, PathBuf },
    fs, io::Write, fmt
};

pub type Coord = i32;

const SAVES_DIRECTORY: &'static str = "saves/";
const WORLD_JSON_FILE: &'static str = "world.json";

#[derive(Debug)]
pub struct World {
    /// The name/title of this world.
    title: String,

    /// Directory containing world data.
    directory: PathBuf,

    /// Seed for random number generation.
    seed: u32,

    /// When this world was created (time since Unix epoch).
    created_timestamp: u64,

    /// When this world was last played (time since Unix epoch).
    last_played_timestamp: u64
}

impl World {
    /// Create a new world with the given title.
    pub fn new(title: String) -> Self {
        let now = time_since_epoch();

        World {
            directory: world_save_directory_path(&title),
            title,
            seed: now as u32,
            created_timestamp: now,
            last_played_timestamp: now
        }
    }

    /// Attempt to load an existing world with the given title from the
    /// filesystem.
    pub fn load(title: String) -> Option<Self> {
        let directory = world_save_directory_path(&title);
        let world_file_path = directory.join(WORLD_JSON_FILE);

        match fs::File::open(&world_file_path) {
            Ok(file) => {
                log::debug!("Opened world JSON file: {}", world_file_path.display());

                match serde_json::from_reader::<fs::File, serde_json::Value>(file) {
                    Ok(json) => {
                        log::debug!("World JSON file data: {}", json);

                        if let Some(version) = json["version"].as_str() {
                            if version == crate::VERSION {
                                log::debug!("World '{}' is version '{}' which matches the game version",
                                            title, version);
                            }
                            else {
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
                        let created_timestamp = json["created"].as_u64().unwrap_or(now);
                        let last_played_timestamp = json["last played"].as_u64().unwrap_or(now);

                        let world = World { title, directory, seed, created_timestamp, last_played_timestamp };

                        log::info!("Loaded world: {}", world);

                        return Some(world);
                    }

                    Err(e) => log::warn!("Failed to parse world JSON file '{}' due to JSON error: {}",
                                         world_file_path.display(), e)
                }
            }

            Err(e) => log::warn!("Failed to read world JSON file '{}' due to IO error: {}",
                                 world_file_path.display(), e)
        }

        None
    }

    /// Save this world to the filesystem.
    pub fn save(&self) -> bool {
        let world_file_path = self.directory.join(WORLD_JSON_FILE);

        let data = serde_json::json!({
            "version": crate::VERSION,
            "seed": self.seed,
            "created": self.created_timestamp,
            "last played": self.last_played_timestamp
        }).to_string();

        log::debug!("Created world JSON data: {}", data);

        match fs::write(&world_file_path, data) {
            Ok(_) => {
                log::info!("Saved world: {}", self);
                true
            }

            Err(e) => {
                log::warn!("Failed to write world JSON file '{}' due to IO error: {}",
                           world_file_path.display(), e);
                false
            }
        }
    }
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}' (directory: '{}', seed: {}, created timestamp: {}, last played timestamp: {})",
               self.title, self.directory.display(), self.seed, self.created_timestamp, self.last_played_timestamp)
    }
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