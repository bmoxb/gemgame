use std::path::{Path, PathBuf};

use shared::maps::{Chunk, ChunkCoords};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{Error, Result};

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
