use std::path::{Path, PathBuf};

use shared::maps::{Chunk, ChunkCoords, Map};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{Error, Result};
use crate::Shared;

/// This function will try the following steps until one succeeds:
/// * Fetch the chunk at the specified coordinates from the given map object's loaded chunks.
/// * Read the chunk at the given coordinates from the filesystem before inserting it into the given map's loaded
///   chunks.
/// * Newly generate a chunk before inserting it into the given map's loaded chunks.
/// Once a chunk is obtained from any of the above steps, it is cloned before being returned from this function.
/// This function is not a method of `super::ServerMap` so that the mutex that that object is contained in is locked for
/// only shortest required period of time.
pub async fn get_or_load_or_generate_chunk(map: &Shared<super::ServerMap>, coords: ChunkCoords) -> Chunk {
    let loaded_chunk_option = map.lock().loaded_chunk_at(coords).cloned();

    if let Some(loaded_chunk) = loaded_chunk_option {
        log::debug!("Chunk at {} already loaded", coords);

        loaded_chunk
    }
    else {
        // Chunk is not already in memory so needs to either be read from disk or newly generated before being loaded
        // into the map.

        let directory = map.lock().directory.clone();

        let new_chunk = load_chunk(&directory, coords).await.unwrap_or_else(|_| {
            let generator = &map.lock().generator;

            log::debug!(
                "Chunk at {} could not be found on disk so will be newly generated using generator '{}'",
                coords,
                generator.name()
            );

            generator.generate(coords)
        });

        // Add the new chunk to map's loaded chunks:
        map.lock().add_chunk(coords, new_chunk.clone());

        new_chunk
    }
}

/// Attempt to asynchronously read data from the file system for the chunk at the specified coordinates.
pub async fn load_chunk(directory: &Path, coords: ChunkCoords) -> Result<Chunk> {
    let chunk_file_path = build_chunk_file_path(directory, coords);

    log::trace!("Attempting to load chunk at {} from file: {}", coords, chunk_file_path.display());

    if let Ok(mut file) = tokio::fs::File::open(&chunk_file_path).await {
        let mut buffer = Vec::new();

        match file.read_to_end(&mut buffer).await {
            Ok(_) => match bincode::deserialize(buffer.as_slice()) {
                Ok(chunk) => {
                    log::debug!("Loaded chunk at {} from file: {}", coords, chunk_file_path.display());

                    Ok(chunk)
                }

                Err(bincode_error) => {
                    log::warn!(
                        "Failed to decode chunk data read from file '{}' - {}",
                        chunk_file_path.display(),
                        bincode_error
                    );

                    Err(Error::EncodingFailure(bincode_error))
                }
            },

            Err(read_error) => {
                log::warn!("Failed to read chunk data from file '{}' - {}", chunk_file_path.display(), read_error);

                Err(Error::AccessFailure(read_error))
            }
        }
    }
    else {
        Err(Error::DoesNotExist(chunk_file_path))
    }
}

/// Attempt to asynchronously write the data comprising the provided chunk to the file system.
pub async fn save_chunk(directory: &Path, coords: ChunkCoords, chunk: &Chunk) -> Result<()> {
    let chunk_file_path = build_chunk_file_path(directory, coords);

    log::trace!("Attempting to save chunk at {} to file: {}", coords, chunk_file_path.display());

    match tokio::fs::File::create(&chunk_file_path).await {
        Ok(mut file) => match bincode::serialize(chunk) {
            Ok(data) => match file.write(data.as_slice()).await {
                Ok(_) => {
                    log::debug!("Saved chunk at {} to file: {}", coords, chunk_file_path.display());
                    Ok(())
                }

                Err(write_error) => {
                    log::warn!("Failed to write chunk data to file '{}' - {}", chunk_file_path.display(), write_error);
                    Err(Error::AccessFailure(write_error))
                }
            },

            Err(bincode_error) => {
                log::warn!("Failed to encode chunk data for {} - {}", coords, bincode_error);
                Err(Error::EncodingFailure(bincode_error))
            }
        },

        Err(create_error) => {
            log::warn!("Failed to create/open chunk data file '{}' - {}", chunk_file_path.display(), create_error);
            Err(Error::AccessFailure(create_error))
        }
    }
}

fn build_chunk_file_path(directory: &Path, coords: ChunkCoords) -> PathBuf {
    let file_name = format!("{}_{}.chunk", coords.x, coords.y);
    directory.join(file_name)
}
