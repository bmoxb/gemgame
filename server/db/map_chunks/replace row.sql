INSERT INTO map_chunks (chunk_x, chunk_y, data)
VALUES ($1, $2, $3)
ON CONFLICT (chunk_x, chunk_y) DO UPDATE
    SET data = $3
