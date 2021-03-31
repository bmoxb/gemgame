//! Includes functions to handle the fetching/saving of player entities from/to the database.

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

use crate::db_query_from_file;

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

    db_query_from_file!("client_entities/create row")
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
    let res = db_query_from_file!("client_entities/select row")
        .bind(client_id.encode())
        .map(|row: sqlx::postgres::PgRow| {
            (
                Id::decode(row.get("entity_id")).unwrap(),
                Entity {
                    pos: TileCoords { x: row.get("tile_x"), y: row.get("tile_y") },
                    direction: Direction::Down,
                    facial_expression: FacialExpression::Neutral,
                    hair_style: decode_variant(row.get("hair_style")),
                    clothing_colour: decode_variant(row.get("clothing_colour")),
                    skin_colour: decode_variant(row.get("skin_colour")),
                    hair_colour: decode_variant(row.get("hair_colour")),
                    has_running_shoes: row.get("has_running_shoes")
                }
            )
        })
        .fetch_optional(db)
        .await;

    res
}

pub async fn update_database_for_player(
    entity: &Entity, client_id: Id, db: &mut sqlx::PgConnection
) -> sqlx::Result<()> {
    db_query_from_file!("client_entities/update row")
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

/// Returns a random variant of the specified enum type.
fn random_variant<T: IntoEnumIterator>() -> T {
    T::iter().choose(&mut rand::thread_rng()).unwrap()
}
