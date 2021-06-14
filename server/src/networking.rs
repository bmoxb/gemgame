use std::convert;

use futures_util::{SinkExt, StreamExt};
use shared::messages;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite, WebSocketStream};

/// Manages a WebSocket connection and simplifies the process of sending and receiving bincode messages.
pub struct Connection {
    ws: WebSocketStream<TcpStream>
}

impl Connection {
    pub async fn new(stream: TcpStream) -> tungstenite::Result<Self> {
        Ok(Connection { ws: tokio_tungstenite::accept_async(stream).await? })
    }

    pub async fn send(&mut self, msg: &messages::FromServer) -> Result<()> {
        let encoded = bincode::serialize(msg)?;
        self.ws.send(tungstenite::Message::Binary(encoded)).await?;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<messages::ToServer>> {
        if let Some(some_result) = self.ws.next().await {
            match some_result? {
                tungstenite::Message::Binary(bytes_vec) => Ok(Some(bincode::deserialize(bytes_vec.as_slice())?)),
                tungstenite::Message::Close(_) => Ok(None),
                not_binary_msg => Err(Error::MessageNotBinary(not_binary_msg))
            }
        }
        else {
            Ok(None)
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        self.ws.close(None).await.map_err(convert::Into::into)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Encoding failed - {0}")]
    EncodingFailure(#[from] bincode::Error),
    #[error("Message is not binary - {0}")]
    MessageNotBinary(tungstenite::Message),
    #[error("Tungstenite error - {0}")]
    Tungstenite(#[from] tungstenite::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
