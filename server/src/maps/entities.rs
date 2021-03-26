use std::convert::TryInto;

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

    // TODO: Randomly select entity features like hair style, skin colour, etc.
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
        "INSERT INTO client_entities (
            client_id, entity_id, tile_x, tile_y, hair_style, clothing_colour, skin_colour, hair_colour, has_running_shoes
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(client_id.encode())
    .bind(entity_id.encode())
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .bind(encode_enum(entity.hair_style))
    .bind(encode_enum(entity.clothing_colour))
    .bind(encode_enum(entity.skin_colour))
    .bind(encode_enum(entity.hair_colour))
    .bind(entity.has_running_shoes)
    .execute(db)
    .await?;

    Ok((entity_id, entity))
}
pub async fn player_from_database(
    client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Any>
) -> sqlx::Result<Option<(Id, Entity)>> {
    let res = sqlx::query(
        "SELECT entity_id, tile_x, tile_y, hair_style, clothing_colour, skin_colour, hair_colour, has_running_shoes
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
                hair_style: decode_enum(row.try_get("hair_style")?, || HairStyle::Quiff), // TODO: Random default.
                clothing_colour: decode_enum(row.try_get("clothing_colour")?, || ClothingColour::Green),
                skin_colour: decode_enum(row.try_get("skin_colour")?, || SkinColour::White),
                hair_colour: decode_enum(row.try_get("hair_colour")?, || HairColour::Green),
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
    entity: &Entity, client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Any>
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE client_entities
        SET tile_x = ?, tile_y = ?, hair_style = ?, clothing_colour = ?, skin_colour = ?, hair_colour = ?, has_running_shoes = ?
        WHERE client_id = ?"
    )
    .bind(entity.pos.x)
    .bind(entity.pos.y)
    .bind(encode_enum(entity.hair_style))
    .bind(encode_enum(entity.clothing_colour))
    .bind(encode_enum(entity.skin_colour))
    .bind(encode_enum(entity.hair_colour))
    .bind(entity.has_running_shoes)
    .bind(client_id.encode())
    .execute(db)
    .await
    .map(|result| {
        let rows_changed = result.rows_affected();
        if rows_changed != 1 {
            log::warn!("Modified {} rows when update player entity data for client with ID {}", rows_changed, client_id);
        }
    })
}

/// Uses Bincode to encode an enum variant as a 32-bit integer.
fn encode_enum<T: serde::Serialize>(val: T) -> i32 {
    i32::from_le_bytes(bincode::serialize(&val).unwrap().try_into().unwrap())
}

/// Decodes a 32-bit integer into a variant of a given enum type via Bincode.
fn decode_enum<T: serde::de::DeserializeOwned>(val: i32, default: impl Fn() -> T) -> T {
    bincode::deserialize(&i32::to_le_bytes(val)).unwrap_or_else(|_| {
        log::warn!("Failed to decode 32-bit integer {} into enum variant of type {}", val, std::any::type_name::<T>());
        default()
    })
}
