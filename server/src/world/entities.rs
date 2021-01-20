use shared::{
    world::{
        entities::{Entity, Variety},
        maps::TileCoords
    },
    Id
};
use sqlx::Row;

/// Represents an entity controlled by a player (i.e. controlled remotedly by a specific client).
pub struct PlayerEntity {
    id: Id,
    contained: Entity,
    current_map_id: Id
}

impl PlayerEntity {
    pub async fn new_to_database(
        client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<Self> {
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

        let current_map_id = Id::new(0); // TODO

        sqlx::query(
            "INSERT INTO client_entities (client_id, entity_id, current_map_id, name, tile_x, tile_y)
            VALUES (?, ?, ?, ?, ?)"
        )
        .bind(client_id.encode())
        .bind(entity_id.encode())
        .bind(current_map_id.encode())
        .bind(&contained.name)
        .bind(contained.pos.x)
        .bind(contained.pos.y)
        .execute(db)
        .await?;

        Ok(PlayerEntity { id: entity_id, contained, current_map_id })
    }

    pub async fn from_database(
        client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<Option<Self>> {
        let res = sqlx::query(
            "SELECT entity_id, current_map_id, name, tile_x, tile_y
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
                },
                Id::decode(row.try_get("current_map_id")?).unwrap() // TODO: Don't just unwrap.
            ))
        })
        .fetch_optional(db)
        .await?
        .transpose();

        res.map(|opt| opt.map(|(id, contained, current_map_id)| PlayerEntity { id, contained, current_map_id }))
    }

    pub async fn update_database(
        &self, client_id: Id, db: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>
    ) -> sqlx::Result<()> {
        sqlx::query(
            "UPDATE client_entities
            SET name, tile_x = ?, tile_y = ?, current_map_id = ?
            WHERE client_id = ?"
        )
        .bind(&self.contained.name)
        .bind(self.contained.pos.x)
        .bind(self.contained.pos.y)
        .bind(self.current_map_id.encode())
        .bind(client_id.encode())
        .execute(db)
        .await
        .map(|_| ())
    }

    pub fn inner_entity_with_id(&self) -> (Id, Entity) { (self.id, self.contained.clone()) }
}

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
