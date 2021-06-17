UPDATE client_entities
SET tile_x = $1, tile_y = $2,
    hair_style = $3, clothing_colour = $4, skin_colour = $5, hair_colour = $6,
    gem_collection = $7, item_inventory = $8, bombs_placed_count = $9
WHERE client_id = $10
