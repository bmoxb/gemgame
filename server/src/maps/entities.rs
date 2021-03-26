use shared::{
    maps::{
        entities::{ClothingColour, Direction, Entity, FacialExpression, HairColour, HairStyle, SkinColour},
        TileCoords
    },
    Id
};
use sqlx::Row;

pub async fn new_player_in_database(
    client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Any>
) -> sqlx::Result<(Id, Entity)> {
    let entity_id = crate::id::generate_with_timestamp();

    // TODO: Randomly select entity features from a list.
    let entity = Entity {
        pos: TileCoords { x: 0, y: 0 }, // TODO: Nearest free position.
        direction: Direction::Down,
        facial_expression: FacialExpression::Neutral,
        hair_style: HairStyle::Quiff,
        clothing_colour: ClothingColour::Red,
        skin_colour: SkinColour::Pale,
        hair_colour: HairColour::Black,
        has_running_shoes: false
    };

    sqlx::query(
        "INSERT INTO client_entities (client_id, entity_id, tile_x, tile_y)
        VALUES (?, ?, ?, ?, ?)"
    )
    .bind(client_id.encode())
    .bind(entity_id.encode())
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .execute(db)
    .await?;
    // TODO: Store entity hair style & colour, skin colour, clothing colour, whether or not they have running shoes.

    Ok((entity_id, entity))
}
pub async fn player_from_database(
    client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Any>
) -> sqlx::Result<Option<(Id, Entity)>> {
    let res = sqlx::query(
        "SELECT entity_id, tile_x, tile_y
        FROM client_entities
        WHERE client_id = ?"
    )
    .bind(client_id.encode())
    .map(|row: sqlx::any::AnyRow| {
        sqlx::Result::Ok((
            Id::decode(row.try_get("entity_id")?).unwrap(), // TODO: Don't just unwrap.
            Entity {
                pos: TileCoords { x: row.try_get("tile_x")?, y: row.try_get("tile_y")? },
                direction: Direction::Down,
                facial_expression: FacialExpression::Neutral,
                hair_style: HairStyle::Quiff, // TODO: From database.
                clothing_colour: ClothingColour::Red,
                skin_colour: SkinColour::Pale,
                hair_colour: HairColour::Black,
                has_running_shoes: false
            }
        ))
    })
    .fetch_optional(db)
    .await?
    .transpose();

    res
}

pub async fn update_database_for_player(
    entity: &Entity, client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Any>
) -> sqlx::Result<()> {
    // TODO: Update name and hair style in database.

    sqlx::query(
        "UPDATE client_entities
        SET tile_x = ?, tile_y = ?
        WHERE client_id = ?"
    )
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .bind(client_id.encode())
    .execute(db)
    .await
    .map(|_| ())
}
