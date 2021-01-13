# Networking Protocol

* All messages between clients and the server are sent via the WebSocket protocol and encoded using Bincode.
* Messages from a client to the server take the form of a variant of the `ToServer` enum while messages from the server to the client use the `FromServer` enum (see the `shared::messages` module).

## Handshake

* The TCP and WebSocket handshakes must be complete upon establishing a connection.
* The client must then send a 'hello' message (`ToServer::Hello` variant). If this the client has played before then they may provide a client ID along with this message (see the following section).
* After receiving a 'hello' message, the server replies with a 'welcome' message (`FromServer::Welcome` variant). If a client ID is provided it will be looked up in the database (see the following section). The 'welcome' message will include the server's version as well as the client's ID and their player entity.

## Returning Clients

* Players may continue their game through a system making use of browser local storage (`window.localStorage.setItem('clientid', ...)`) or filesystem storage when playing via the desktop application (stored in a text file simply called `clientid.txt`).
* When a client connects without providing an existing client ID, the sever generates a new ID and a new player entity. These are then inserted into the database before being returned to the player.
* A client can connect and provide a client ID to the server. If that client ID is found in the database, the corresponding entity is returned to the client. Otherwise, the provided ID is discarded and the server treats the client as if it were a new one.
* Whenever a returning client connects, the server updates their corresponding database record with the current time. This is done so that records for players who go some amount of time without playing (six months maybe?) can be removed from the database.
