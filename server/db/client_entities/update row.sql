UPDATE client_entities
SET tile_x = $1, tile_y = $2,
    hair_style = $3, clothing_colour = $4, skin_colour = $5, hair_colour = $6,
    has_running_shoes = $7,
    emerald_quantity = $8, ruby_quantity = $9, diamond_quantity = $10
WHERE client_id = $11
