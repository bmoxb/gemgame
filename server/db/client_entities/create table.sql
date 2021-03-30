CREATE TABLE IF NOT EXISTS client_entities (
    client_id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL UNIQUE,
    tile_x INTEGER NOT NULL,
    tile_y INTEGER NOT NULL,
    hair_style SMALLINT NOT NULL,
    clothing_colour SMALLINT NOT NULL,
    skin_colour SMALLINT NOT NULL,
    hair_colour SMALLINT NOT NULL,
    has_running_shoes BOOLEAN NOT NULL
)
