use std::convert;

use shared::maps::{Chunk, ChunkCoords, Map};
use sqlx::Row;

use crate::{db_query_from_file, Shared};

/// This function will try the following steps until one succeeds:
/// * Fetch the chunk at the specified coordinates from the given map object's loaded chunks.
/// * Read the chunk at the given coordinates from the filesystem before inserting it into the given map's loaded
///   chunks.
/// * Newly generate a chunk before inserting it into the given map's loaded chunks.
/// Once a chunk is obtained from any of the above steps, it is cloned before being returned from this function.
/// This function is not a method of `super::ServerMap` so that the mutex that that object is contained in is locked for
/// only shortest required period of time.
pub async fn get_or_load_or_generate_chunk(
    db: sqlx::pool::PoolConnection<sqlx::Postgres>, map: &Shared<super::ServerMap>, coords: ChunkCoords
) -> Chunk {
    let loaded_chunk_option = map.lock().loaded_chunk_at(coords).cloned();

    if let Some(loaded_chunk) = loaded_chunk_option {
        log::debug!("Chunk at {} already loaded", coords);

        loaded_chunk
    }
    else {
        // Chunk is not already in memory so needs to either be fetched from the database or newly generated before
        // being loaded into the map.

        let new_chunk = load_chunk(db, coords).await.unwrap_or_else(|_| {
            let generator = &map.lock().generator;

            log::debug!(
                "Chunk at {} could not be loaded from the database so will be newly generated using generator '{}'",
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
pub async fn load_chunk(mut db: sqlx::pool::PoolConnection<sqlx::Postgres>, coords: ChunkCoords) -> Result<Chunk> {
    log::trace!("Attempting to load chunk at {} from database", coords);

    let res = db_query_from_file!("map_chunks/select row")
        .bind(coords.x)
        .bind(coords.y)
        .map(|row| {
            let data: &[u8] = row.get("data");
            Ok(bincode::deserialize(data)?)
        })
        .fetch_one(&mut db)
        .await?;

    log::debug!("Successfully loaded chunk at {} from database", coords);

    res
}

/// Attempt to asynchronously write the data comprising the provided chunk to the file system.
pub async fn save_chunk(
    mut db: sqlx::pool::PoolConnection<sqlx::Postgres>, coords: ChunkCoords, chunk: &Chunk
) -> Result<()> {
    log::trace!("Attempting to save chunk at {} to database", coords);

    db_query_from_file!("map_chunks/replace row")
        .bind(coords.x)
        .bind(coords.y)
        .bind(bincode::serialize(chunk)?)
        .execute(&mut db)
        .await
        .map(|_| {
            log::debug!("Successfully wrote chunk at {} to database", coords);
        })
        .map_err(convert::Into::into) // Map error type.
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to access database")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to (de)serialise chunk data with Bincode")]
    BincodeError(#[from] bincode::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
