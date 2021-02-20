# Server Design

* The server is to be built on top of Tokio using Tungstenite for handling WebSocket connections.
* The server features a simple main loop that listens for incoming TCP/IP connections. When a connection is received, a new Tokio task is spawned to handle that connection.

## Connection Task

* Once a TCP/IP connection is established with a client, a dedicated Tokio task is created to handle it.
* The first duty of each connection task is to perform the WebSocket handshake with the client and then the exchange of 'hello' and 'welcome' messages (see the protocol document also in the `design/` directory).

## Tracking World Changes

* The game world is stored and shared between all connection tasks/threads using a `RwLock` wrapped inside of an `Arc`.
* Each connection task must keep track of which map chunks its corresponding connection has loaded. This is done based via the `ToServer::RequestChunk` and `ToServer::ChunkUnloadedLocally` messages.
* In addition to polling the WebSocket connection, each task must also poll the Tokio broadcast channel in order to check for changes to the game world. If those changes are relevant to that task's client (i.e. they're changes to chunks that that client has loaded) then that task's client must be sent messages via the WebSocket connection informing them of said changes.
* Whenever a task wishes to modify the game world, it must do two things:
  * Lock the game world `RwLock` for writing and make the desired changes.
  * Send message(s) on the Tokio broadcast channel in order to ensure all other tasks are immediately notified of changes.

