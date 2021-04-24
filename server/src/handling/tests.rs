#![cfg(test)]

use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc
};

use parking_lot::Mutex;
use shared::{
    gems, items,
    maps::{
        entities::{ClothingColour, Direction, Entity, FacialExpression, HairColour, HairStyle, SkinColour},
        Chunk, ChunkCoords, OffsetCoords, Tile, TileCoords, CHUNK_WIDTH
    }
};

use super::*;

async fn make_test_handler() -> Handler {
    let (map_changes_sender, map_changes_receiver) = broadcast::channel(5);

    super::Handler {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        db_pool: sqlx::postgres::PgPoolOptions::new().connect_lazy("postgres://").unwrap(),
        game_map: Arc::new(Mutex::new(ServerMap::new_with_default_generator(0))),
        map_changes_sender,
        map_changes_receiver,
        remote_loaded_chunk_coords: Vec::new()
    }
}

impl Handler {
    fn add_test_entity(&mut self, pos: TileCoords) -> Id {
        let entity_id = crate::id::generate_with_timestamp();

        let entity = Entity {
            pos,
            direction: Direction::Down,
            facial_expression: FacialExpression::Neutral,
            hair_style: HairStyle::Quiff,
            clothing_colour: ClothingColour::Grey,
            skin_colour: SkinColour::Black,
            hair_colour: HairColour::Black,
            has_running_shoes: false,
            gem_collection: gems::Collection::default(),
            item_inventory: items::Inventory::default()
        };
        self.game_map.lock().add_entity(entity_id, entity);

        entity_id
    }

    fn add_empty_chunk(&mut self, coords: ChunkCoords) {
        self.add_chunk(coords, Chunk::default());
    }

    fn add_chunk(&mut self, coords: ChunkCoords, chunk: Chunk) {
        self.game_map.lock().add_chunk(coords, chunk);
        self.remote_loaded_chunk_coords.push(coords);
    }
}

/// Ensure that no response is provided when an unexpected (i.e. sent after connection establishment) 'hello' message is
/// recevied.
#[tokio::test(flavor = "multi_thread")]
async fn handle_unexpected_hello_msg() {
    let mut handler = make_test_handler().await;

    let id = crate::id::generate_random();
    let msg = messages::ToServer::Hello { client_id_option: None };

    assert!(handler.handle_message(msg, id).await.unwrap().is_empty());
}

/// Ensure that a 'move my entity' message causes the entity's position on the game map to be appropriately updated, a
/// response message is created, and that a message on the map changes broadcast channel is sent to inform other tasks
/// of the change.
#[tokio::test(flavor = "multi_thread")]
async fn handle_move_my_entity() {
    let mut handler = make_test_handler().await;
    let mut other_map_changes_receiver = handler.map_changes_sender.subscribe();

    handler.add_empty_chunk(ChunkCoords { x: 0, y: 0 });
    let player_id = handler.add_test_entity(TileCoords { x: 5, y: 5 });

    let msg = messages::ToServer::MoveMyEntity { request_number: 0, direction: Direction::Right };
    let responses = handler.handle_message(msg, player_id).await.unwrap();

    // Ensure the response message is correct:
    assert_eq!(responses.len(), 1);
    assert!(matches!(
        responses[0],
        messages::FromServer::YourEntityMoved { request_number: 0, new_position: TileCoords { x: 6, y: 5 } }
    ));

    // Ensure the entity's position was changed appropriately:
    assert_eq!(handler.game_map.lock().entity_by_id(player_id).unwrap().pos, TileCoords { x: 6, y: 5 });

    // A message should not be broadcast to this task's map changes recevier:
    assert!(matches!(handler.map_changes_receiver.try_recv(), Err(broadcast::error::TryRecvError::Empty)));

    // A message should have been sent to all the map changes receviers of other tasks:
    let change = other_map_changes_receiver.recv().await.unwrap();

    assert!(matches!(
        change,
        maps::Modification::EntityMoved {
            entity_id,
            old_position: TileCoords { x: 5, y: 5 },
            new_position: TileCoords { x: 6, y: 5 },
            direction: _
        } if entity_id == player_id
    ));
}

/// Ensure that a 'move my entity' message that would fail due to a blocking tile or entity being in the way does
/// not modify the player entity's position, and does *not* send a message on the map modifications channel. Also
/// ensures that the client is sent a message informing them that their entity movement could not go ahead.
#[tokio::test(flavor = "multi_thread")]
async fn handle_move_my_entity_blocking() {
    let mut handler = make_test_handler().await;
    let mut other_map_changes_receiver = handler.map_changes_sender.subscribe();

    let mut chunk = Chunk::default();
    chunk.set_tile_at_offset(OffsetCoords { x: 0, y: 1 }, Tile::Rock);

    handler.add_chunk(ChunkCoords { x: 0, y: 0 }, chunk);

    let player_starting_coords = TileCoords { x: 0, y: 0 };
    let player_id = handler.add_test_entity(player_starting_coords);

    let msg = messages::ToServer::MoveMyEntity { request_number: 0, direction: Direction::Down };
    let responses = handler.handle_message(msg, player_id).await.unwrap();

    // Response message informing the entity that they could not perform such a movement should have been returned:
    assert_eq!(responses.len(), 1);
    assert!(matches!(
        responses[0],
        messages::FromServer::YourEntityMoved { request_number: 0, new_position } if new_position == player_starting_coords
    ));

    // Ensure player entity's position did not change:
    assert_eq!(handler.game_map.lock().entity_by_id(player_id).unwrap().pos, player_starting_coords);

    // No message should have been sent on the map changes channel:
    assert!(matches!(other_map_changes_receiver.try_recv(), Err(broadcast::error::TryRecvError::Empty)));
}

/// Ensure that a task produces an entity moved message to send to its remote client when it receives an entity moved
/// message on the map modification broadcast channel, provided the entity in question moved within one of the remote
/// client's loaded chunks.
#[tokio::test(flavor = "multi_thread")]
async fn handle_entity_moved_within_loaded_chunk() {
    let mut handler = make_test_handler().await;

    handler.add_empty_chunk(ChunkCoords { x: 0, y: 0 });
    let entity_id = handler.add_test_entity(TileCoords { x: 5, y: 5 });

    let modification = maps::Modification::EntityMoved {
        entity_id,
        old_position: TileCoords { x: 5, y: 5 },
        new_position: TileCoords { x: 6, y: 5 },
        direction: Direction::Right
    };

    assert!(matches!(
        handler.handle_map_change(modification).await.unwrap(),
        messages::FromServer::MoveEntity(id, TileCoords { x: 6, y: 5 }, _) if id == entity_id
    ));
}

/// Ensure a provide entity message is produced when an entity moved broadcast message is received for an entity moving
/// into one of this task's remote client's loaded chunks.
#[tokio::test(flavor = "multi_thread")]
async fn handle_entity_moved_into_loaded_chunk() {
    let mut handler = make_test_handler().await;

    handler.add_empty_chunk(ChunkCoords { x: 0, y: 0 });
    let entity_id = handler.add_test_entity(TileCoords { x: CHUNK_WIDTH, y: 5 });

    let modification = maps::Modification::EntityMoved {
        entity_id,
        old_position: TileCoords { x: CHUNK_WIDTH, y: 5 }, // chunk at 1, 0
        new_position: TileCoords { x: CHUNK_WIDTH - 1, y: 5 }, // chunk at 0, 0
        direction: Direction::Left
    };

    assert!(matches!(
        handler.handle_map_change(modification).await.unwrap(),
        messages::FromServer::ProvideEntity(id, _) if id == entity_id
    ));
}

/// Ensure that an unload entity message is produced when an entity moved broadcast message is received for an entity
/// moving out of one of this task's remote client's loaded chunks.
#[tokio::test(flavor = "multi_thread")]
async fn handle_entity_moved_leaving_loaded_chunk() {
    let mut handler = make_test_handler().await;

    handler.add_empty_chunk(ChunkCoords { x: 0, y: 0 });
    let entity_id = handler.add_test_entity(TileCoords { x: 0, y: 5 });

    let modification = maps::Modification::EntityMoved {
        entity_id,
        old_position: TileCoords { x: 0, y: 5 },
        new_position: TileCoords { x: -1, y: 5 },
        direction: Direction::Left
    };

    assert!(matches!(
        handler.handle_map_change(modification).await.unwrap(),
        messages::FromServer::ShouldUnloadEntity(id) if id == entity_id
    ));
}

/// Ensure that no message to be sent to this task's remote client is produced when an entity moved broadcast message is
/// received for an entity moving entirely outside of the remote client's loaded chunks.
#[tokio::test(flavor = "multi_thread")]
async fn handle_entity_moved_outside_loaded_chunk() {
    let mut handler = make_test_handler().await;

    let entity_id = handler.add_test_entity(TileCoords { x: 12, y: 13 });

    let modification = maps::Modification::EntityMoved {
        entity_id,
        old_position: TileCoords { x: 12, y: 13 },
        new_position: TileCoords { x: 13, y: 13 },
        direction: Direction::Left
    };

    assert!(handler.handle_map_change(modification).await.is_none());
}

/// Ensure that a task produces a provide entity message to send to its remote client when it is informed via the map
/// changes broadcast channel of a new entity being added to the map within the remote client's loaded chunks.
#[tokio::test(flavor = "multi_thread")]
async fn handle_entity_added_within_loaded_chunks() {
    let mut handler = make_test_handler().await;

    handler.add_empty_chunk(ChunkCoords { x: 0, y: 0 });
    let entity_id = handler.add_test_entity(TileCoords { x: 5, y: 5 });

    let modification = maps::Modification::EntityAdded(entity_id);

    assert!(matches!(
        handler.handle_map_change(modification).await.unwrap(),
        messages::FromServer::ProvideEntity(id, _) if id == entity_id
    ));
}

/// Ensure that a task produces an unloaded entity message to send to its remote client when it is informed via the map
/// changes broadcast channel of an existing entity being removed from the map within the remote client's loaded chunks.
#[tokio::test(flavor = "multi_thread")]
async fn handle_entity_removed_within_loaded_chunks() {
    let mut handler = make_test_handler().await;

    handler.add_empty_chunk(ChunkCoords { x: 0, y: 0 });
    let entity_id = handler.add_test_entity(TileCoords { x: 5, y: 5 });

    let modification = maps::Modification::EntityRemoved(entity_id, ChunkCoords { x: 0, y: 0 });

    assert!(matches!(
        handler.handle_map_change(modification).await.unwrap(),
        messages::FromServer::ShouldUnloadEntity(id) if id == entity_id
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn handle_smashed_rock_within_loaded_chunks() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn handle_smashed_rock_outside_loaded_chunks() {
    // TODO
}
