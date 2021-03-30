UPDATE client_entities
SET tile_x = $1, tile_y = $2,
    hair_style = $3, clothing_colour = $4, skin_colour = $5, hair_colour = $6,
    has_running_shoes = $7
WHERE client_id = $8
