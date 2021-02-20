use shared::{
    maps::{
        entities::{Entity, Variety},
        TileCoords
    },
    Id
};
use sqlx::Row;

pub async fn new_player_in_database(
    client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
) -> sqlx::Result<(Id, Entity)> {
    let entity_id = crate::id::generate_with_timestamp();

    let entity = Entity {
        name: "unnamed".to_string(),
        pos: TileCoords::default(),
        variety: Variety::Human {
            direction: Default::default(),
            facial_expression: Default::default(),
            hair_style: Default::default()
        }
    };

    sqlx::query(
        "INSERT INTO client_entities (client_id, entity_id, name, tile_x, tile_y)
        VALUES (?, ?, ?, ?, ?)"
    )
    .bind(client_id.encode())
    .bind(entity_id.encode())
    .bind(&entity.name)
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .execute(db)
    .await?;

    Ok((entity_id, entity))
}

pub async fn player_from_database(
    client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
) -> sqlx::Result<Option<(Id, Entity)>> {
    let res = sqlx::query(
        "SELECT entity_id, name, tile_x, tile_y
        FROM client_entities
        WHERE client_id = ?"
    )
    .bind(client_id.encode())
    .map(|row| {
        sqlx::Result::Ok((
            Id::decode(row.try_get("entity_id")?).unwrap(), // TODO: Don't just unwrap.
            Entity {
                name: row.try_get("name")?,
                pos: TileCoords { x: row.try_get("tile_x")?, y: row.try_get("tile_y")? },
                variety: Variety::Human {
                    direction: Default::default(),
                    facial_expression: Default::default(),
                    hair_style: Default::default() // TODO: From database.
                }
            }
        ))
    })
    .fetch_optional(db)
    .await?
    .transpose();

    res
}

pub async fn update_database_for_player(
    entity: &Entity, client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE client_entities
        SET name = ?, tile_x = ?, tile_y = ?
        WHERE client_id = ?"
    )
    .bind(&entity.name)
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .bind(client_id.encode())
    .execute(db)
    .await
    .map(|_| ())
}
