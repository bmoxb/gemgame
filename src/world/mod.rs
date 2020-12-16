mod entities;
mod maps;
pub mod rendering;

use std::{
    path::{ Path, PathBuf },
    time::SystemTime,
    rc::Rc,
    fs, fmt
};

use raylib::prelude::*;

use entities::Entity;

use maps::Map;

pub type Coord = i32;

const SAVES_DIRECTORY: &'static str = "saves/";
const WORLD_JSON_FILE: &'static str = "world.json";

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
    last_played_timestamp: u64,

    /// The current loaded map.
    current_map: Map,

    /// The player entity.
    player: Entity,

    turn_of_player: bool
}

impl World {
    /// Create a new world with the given title.
    pub fn new(title: String) -> Self {
        let directory = world_save_directory_path(&title);
        let now = time_since_epoch();
        let seed = now as u32;

        World {
            current_map: Map::new(
                directory.join("surface/"),
                Box::new(maps::generators::SurfaceGenerator::new(seed))
            ),
            title, directory, seed,
            created_timestamp: now,
            last_played_timestamp: now,
            player: Entity::new(Rc::new(entities::PlayerController {}), 0, 0),
            turn_of_player: true
        }
    }

    /// Attempt to load an existing world with the given title from the
    /// filesystem. This method relies on the [`load_json`] helper function.
    pub fn load(title: String) -> Option<Self> {
        load_json("world", world_save_directory_path(&title), WORLD_JSON_FILE, |json, directory| {
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

            let map_name = json["current map"].as_str().unwrap_or("surface/");
            let map_directory = directory.join(map_name);

            let current_map = match Map::load(map_directory.clone(), seed) {
                Some(map) => map,
                None => {
                    log::warn!("Specified current map '{}' could not be found so it will now be newly generated",
                                map_name);

                    Map::new(map_directory, Box::new(maps::generators::SurfaceGenerator::new(seed)))
                }
            };

            let turn_of_player = json["player's turn"].as_bool().unwrap_or(true);

            World {
                title, directory, seed,
                created_timestamp, last_played_timestamp,
                current_map,
                player: Entity::new(Rc::new(entities::PlayerController {}), 0, 0), // TODO: Load player!
                turn_of_player
            }
        })
    }

    /// Save this world and its current map to the filesystem. Will return `true`
    /// if able to save the data successfully.
    pub fn save(&self) -> bool {
        let world_file_path = self.directory.join(WORLD_JSON_FILE);

        let data = serde_json::json!({
            "version": crate::VERSION,
            "seed": self.seed,
            "created": self.created_timestamp,
            "last played": self.last_played_timestamp,
            "player's turn": self.turn_of_player
        }).to_string();

        log::debug!("Created world JSON data: {}", data);

        match fs::write(&world_file_path, data) {
            Ok(_) => {
                log::info!("Saved world: {}", self);

                self.current_map.save()
            }

            Err(e) => {
                log::warn!("Failed to write world JSON file '{}' due to IO error: {}",
                           world_file_path.display(), e);
                false
            }
        }
    }

    pub fn update(&mut self, handle: &RaylibHandle) {
        if self.turn_of_player {
            let completed_turn = self.player.your_turn(&mut self.current_map, handle);
            self.turn_of_player = !completed_turn;
        }
        else {
            self.current_map.have_entities_take_their_turns(handle);
            log::info!("Non-player entities have completed their turns - waiting for player...");
            self.turn_of_player = true;
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

/// Helper function that exists to reduce the repetition of code between the
/// [`World::load`] and [`Map::load`] functions.
fn load_json<T: fmt::Display>(result_name: &'static str,
                              directory: PathBuf, file_name: &'static str,
                              success_callback: impl FnOnce(serde_json::Value, PathBuf) -> T) -> Option<T> {
    let file_path = directory.join(file_name);

    match fs::File::open(&file_path) {
        Ok(file) => {
            log::debug!("Opened {} JSON file: {}", result_name, file_path.display());

            match serde_json::from_reader::<fs::File, serde_json::Value>(file) {
                Ok(json) => {
                    log::debug!("JSON file data for this {}: {}", result_name, json);

                    let result = success_callback(json, directory);

                    log::info!("Loaded {}: {}", result_name, result);

                    return Some(result);
                }

                Err(e) => log::warn!("Failed to parse {} JSON file '{}' due to JSON error: {}",
                                     result_name, file_path.display(), e)
            }
        }

        Err(e) => log::warn!("Failed to read {} JSON file '{}' due to IO error: {}",
                             result_name, file_path.display(), e)
    }

    None
}

#[cfg(test)]
mod test {
    // TODO: ...
}