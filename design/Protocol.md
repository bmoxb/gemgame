# Network Protocol

* All messages between clients and the server are sent via the WebSocket protocol and encoded using Bincode.
* Messages from a client to the server take the form of a variant of the `ToServer` enum while messages from the server to the client use the `FromServer` enum (see the `messages` module in the `shared` library).

## Handshake

* The TCP and WebSocket handshakes must be complete upon establishing a connection.
* The client must then send a 'hello' message (`ToServer::Hello` variant). If this the client has played before then they may provide a client ID along with this message (see the following section).
* After receiving a 'hello' message, the server replies with a 'welcome' message (`FromServer::Welcome` variant). If a client ID is provided it will be looked up in the database (see the following section). The 'welcome' message will include the server's version as well as the client's ID and their player entity.

## Returning Clients

* Players may continue their game through a system making use of browser local storage (stored using `window.localStorage.setItem('clientid', ...)`) or filesystem storage when playing via the desktop application (stored in a text file simply called `clientid.txt`).
* When a client connects without providing an existing client ID, the sever generates a new ID and a new player entity. These are then inserted into the database before being returned to the player.
* A client can connect and provide a client ID to the server. If that client ID is found in the database, the corresponding entity is returned to the client. Otherwise, the provided ID is discarded and the server treats the client as if it were a new one.
* Whenever a returning client connects, the server updates their corresponding database record with the current time. This is done so that records for players who go some amount of time without playing (six months maybe?) can be removed from the database.

## Player Movement

* A client can move its player entity by sending a `ToServer::MoveMyEntity { request_number, direction }` message to the server.
* The client should check if the their player can actually move in the specified direction first (i.e. no blocking tiles or entities in the way).
* The client should then, immediately after sending a `MoveMyEntity` message, play the movement animation and locally update the player entity's position without waiting for the server's response.
* The client should incrementally number each movement message sent to the server (provide with `request_number` field) as well as keep track of what it expects the position of the player entity to be after each movement.
* The client should only send a new `MoveMyEntity` message after any on-going movement animations are complete. Note that rapidly sending movement messages will not allow a player to move any quicker as movement speed is limited on the server side so as to prevent cheating.
* When the server receives a `MoveMyEntity` message it must perform a few checks before responding with a `FromServer::YourEntityMoved { request_number, new_position }` message.
* The server must keep track of the last point in time that each player entity moved so as to prevent cheaters from modifying their client to send many `MoveMyEntity` messages in an effort to move quicker than other players. If the server receives a `MoveMyEntity` message from a client earlier than expected/allowed then the movement should be queued to run as soon as the required amount of time has passed.
* The server should not trust the client to only send valid movements and should therefore check that the direction the client wishes to move in is clear of blocking tiles and other entities. If it is, the client's player entity's coordinates should be updated accordingly.
* When the server thread/task changes a player entity's coordinates it should update all other tasks of that change using the world modification multi-sender, multi-receiver channels so that those tasks may inform their respective remote clients as necessary (using `FromServer::EntityMoved` messages).
* The server should include the same `request_number` value with its `YourEntityMoved` response message as was included in the `MoveMyEntity` message that triggered the movement process. This is so that the client may ensure that each prediction of the server's response made was correct. If a client finds that the position it believes its player entity would be at for a given `request_number` differs from the position specified by the received `YourEntityMoved` message, it should disregard its prediction and locally set the entity's position to that specified by the server.
