INSERT INTO client_entities (
    tile_x, tile_y,
    hair_style, clothing_colour, skin_colour, hair_colour,
    gem_collection, item_inventory,
    client_id, entity_id
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
