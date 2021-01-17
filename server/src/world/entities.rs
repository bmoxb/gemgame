use shared::{
    world::{entities::Entity, maps::TileCoords},
    Id
};
use sqlx::Row;

/// Represents an entity controlled by a player (i.e. controlled remotedly by a specific client).
pub struct PlayerEntity {
    contained: Entity,
    current_map_id: Id
}

impl PlayerEntity {
    pub async fn new_to_database(
        client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<Self> {
        let contained = Entity {
            id: crate::id::generate_with_timestamp(),
            pos: TileCoords::default(),
            direction: Default::default()
        };
        let current_map_id = Id::new(0); // TODO

        sqlx::query(
            "INSERT INTO entities (entity_id, current_map_id, tile_x, tile_y) VALUES (?, ?, ?, ?);
            INSERT INTO clients (client_id, entity_id) VALUES (?, ?)"
        )
        .bind(contained.id.encode())
        .bind(current_map_id.encode())
        .bind(contained.pos.x)
        .bind(contained.pos.y)
        .bind(client_id.encode())
        .bind(contained.id.encode())
        .execute(db)
        .await?;

        Ok(PlayerEntity { contained, current_map_id })
    }

    pub async fn from_database(
        client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<Option<Self>> {
        let res = sqlx::query(
            "SELECT entities.entity_id, current_map_id, tile_x, tile_y
            FROM entities INNER JOIN clients ON entities.entity_id = clients.entity_id
            WHERE clients.client_id = ?"
        )
        .bind(client_id.encode())
        .map(|row| {
            sqlx::Result::Ok((
                Entity {
                    id: Id::decode(row.try_get("entity_id")?).unwrap(), // TODO
                    pos: TileCoords { x: row.try_get("tile_x")?, y: row.try_get("tile_y")? },
                    direction: Default::default() // TODO
                },
                Id::decode(row.try_get("current_map_id")?).unwrap() // TODO
            ))
        })
        .fetch_optional(db)
        .await?
        .transpose();

        res.map(|opt| opt.map(|(contained, current_map_id)| PlayerEntity { contained, current_map_id }))
    }

    pub async fn update_database(
        &self, client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<()> {
        sqlx::query(
            "UPDATE entities SET tile_x = ?, tile_y = ?, current_map_id = ?
            WHERE entity_id = (SELECT entity_id FROM clients WHERE client_id = ?)"
        )
        .bind(self.contained.pos.x)
        .bind(self.contained.pos.y)
        .bind(self.current_map_id.encode())
        .bind(client_id.encode())
        .execute(db)
        .await
        .map(|_| ())
    }

    pub fn inner_entity_cloned(&self) -> Entity { self.contained.clone() }
}

/// Represents an entity controlled by the server.
pub struct NonPlayerEntity {
    contained: Entity,
    controller: Box<dyn Controller>
}

impl NonPlayerEntity {
    fn update(&mut self) { self.controller.update(&mut self.contained); }
}

pub trait Controller {
    fn update(&mut self, e: &mut Entity);
}
