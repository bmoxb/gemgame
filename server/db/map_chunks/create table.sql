CREATE TABLE IF NOT EXISTS map_chunks (
    chunk_x INTEGER NOT NULL,
    chunk_y INTEGER NOT NULL,
    bincode_data BYTEA,
    PRIMARY KEY (chunk_x, chunk_y)
)
