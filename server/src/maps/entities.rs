use shared::{
    maps::{
        entities::{Direction, Entity, Variety},
        TileCoords
    },
    Id
};
use sqlx::Row;

/// Represents an entity controlled by a player (i.e. controlled remotely by a specific client).
pub struct PlayerEntity {
    contained: Entity
}

impl PlayerEntity {
    pub async fn new_to_database(
        client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<(Id, Self)> {
        let entity_id = crate::id::generate_with_timestamp();

        let contained = Entity {
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
        .bind(&contained.name)
        .bind(contained.pos.x)
        .bind(contained.pos.y)
        .execute(db)
        .await?;

        Ok((entity_id, PlayerEntity { contained }))
    }

    pub async fn from_database(
        client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<Option<(Id, Self)>> {
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

        res.map(|opt| opt.map(|(id, contained)| (id, PlayerEntity { contained })))
    }

    pub async fn update_database(
        &self, client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<()> {
        sqlx::query(
            "UPDATE client_entities
            SET name = ?, tile_x = ?, tile_y = ?
            WHERE client_id = ?"
        )
        .bind(&self.contained.name)
        .bind(self.contained.pos.x)
        .bind(self.contained.pos.y)
        .bind(client_id.encode())
        .execute(db)
        .await
        .map(|_| ())
    }

    pub fn position(&self) -> TileCoords { self.contained.pos }

    /// Modify entity position without performing any sort of checks.
    pub fn move_towards_unchecked(&mut self, direction: Direction) {
        let new_pos = direction.apply(self.contained.pos);
        self.contained.pos = new_pos;
    }

    pub fn inner_entity_cloned(&self) -> Entity { self.contained.clone() }
}

/*
/// Represents an entity controlled by the server.
pub struct NonPlayerEntity {
    contained: Entity,
    controller: Box<dyn Controller>
}

impl NonPlayerEntity {
    pub fn update(&mut self) { self.controller.update(&mut self.contained); }
}

pub trait Controller {
    fn update(&mut self, e: &mut Entity);
}
*/
