INSERT INTO client_entities (
    client_id, entity_id,
    tile_x, tile_y,
    hair_style, clothing_colour, skin_colour, hair_colour,
    has_running_shoes
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
