use rand::seq::IteratorRandom;
use shared::{
    maps::{
        entities::{Direction, Entity, FacialExpression},
        TileCoords
    },
    Id
};
use sqlx::Row;
use strum::IntoEnumIterator;

pub async fn new_player_in_database(client_id: Id, db: &mut sqlx::PgConnection) -> sqlx::Result<(Id, Entity)> {
    let entity_id = crate::id::generate_with_timestamp();

    let entity = Entity {
        pos: TileCoords { x: 0, y: 0 }, // TODO: Nearest free position.
        direction: Direction::Down,
        facial_expression: FacialExpression::Neutral,
        hair_style: random_variant(),
        clothing_colour: random_variant(),
        skin_colour: random_variant(),
        hair_colour: random_variant(),
        has_running_shoes: false
    };

    sqlx::query(
        "INSERT INTO client_entities (
            client_id, entity_id, tile_x, tile_y, hair_style, clothing_colour, skin_colour, hair_colour,
            has_running_shoes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);"
    )
    .bind(client_id.encode())
    .bind(entity_id.encode())
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .bind(encode_variant(entity.hair_style))
    .bind(encode_variant(entity.clothing_colour))
    .bind(encode_variant(entity.skin_colour))
    .bind(encode_variant(entity.hair_colour))
    .bind(entity.has_running_shoes)
    .execute(db)
    .await?;

    Ok((entity_id, entity))
}

pub async fn player_from_database(client_id: Id, db: &mut sqlx::PgConnection) -> sqlx::Result<Option<(Id, Entity)>> {
    let res = sqlx::query("SELECT * FROM client_entities WHERE client_id = $1")
        .bind(client_id.encode())
        .map(|row: sqlx::postgres::PgRow| {
            sqlx::Result::Ok((
                Id::decode(row.try_get("entity_id")?).unwrap(), // TODO: Don't just unwrap.
                Entity {
                    pos: TileCoords { x: row.try_get("tile_x")?, y: row.try_get("tile_y")? },
                    direction: Direction::Down,
                    facial_expression: FacialExpression::Neutral,
                    hair_style: decode_variant(row.try_get("hair_style")?),
                    clothing_colour: decode_variant(row.try_get("clothing_colour")?),
                    skin_colour: decode_variant(row.try_get("skin_colour")?),
                    hair_colour: decode_variant(row.try_get("hair_colour")?),
                    has_running_shoes: row.try_get("has_running_shoes")?
                }
            ))
        })
        .fetch_optional(db)
        .await?
        .transpose();

    res
}

pub async fn update_database_for_player(
    entity: &Entity, client_id: Id, db: &mut sqlx::PgConnection
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE client_entities
        SET tile_x = $1, tile_y = $2, hair_style = $3, clothing_colour = $4, skin_colour = $5, hair_colour = $6,
            has_running_shoes = $7
        WHERE client_id = $8"
    )
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .bind(encode_variant(entity.hair_style))
    .bind(encode_variant(entity.clothing_colour))
    .bind(encode_variant(entity.skin_colour))
    .bind(encode_variant(entity.hair_colour))
    .bind(entity.has_running_shoes)
    .bind(client_id.encode())
    .execute(db)
    .await
    .map(|result| {
        let rows_changed = result.rows_affected();
        if rows_changed != 1 {
            log::warn!(
                "Modified {} rows when update player entity data for client with ID {}",
                rows_changed,
                client_id
            );
        }
    })
}

/// Encode an enum variant as a 16-bit integer.
fn encode_variant<T: IntoEnumIterator + PartialEq>(val: T) -> i16 {
    T::iter().position(|x| x == val).unwrap() as i16
}

/// Decodes a 16-bit integer into a variant of a given enum type. If the given integer does not corespond to a variant
/// of the given enum type, then a random variant is returned and a warning message is printed.
fn decode_variant<T: IntoEnumIterator>(val: i16) -> T {
    T::iter().nth(val as usize).unwrap_or_else(|| {
        log::warn!("Failed to decode 32-bit integer {} into enum variant of type {}", val, std::any::type_name::<T>());
        random_variant()
    })
}

fn random_variant<T: IntoEnumIterator>() -> T {
    T::iter().choose(&mut rand::thread_rng()).unwrap()
}
