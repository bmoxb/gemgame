use super::{ Error, Result };

use std::{ convert, thread, sync::mpsc };

pub struct PendingConnection {
    thread_receiver: mpsc::Receiver<Result<Connection>>
}

impl super::PendingConnection<Connection> for PendingConnection {
    fn new(url: &str) -> Self {
        let url = websocket::client::Url::parse(url).expect("WebSocket URL is invalid");

        let (thread_sender, thread_receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut builder = websocket::ClientBuilder::from_url(&url);

            let result = builder.connect(None)
                         .or(Err(Error::ConnectionError))
                         .and_then(|mut ws| {
                ws.set_nodelay(true)?;
                ws.set_nonblocking(true)?;

                Ok(Connection { ws })
            });

            thread_sender.send(result).unwrap();
        });

        PendingConnection { thread_receiver }
    }

    fn ready(&self) -> Result<Connection> {
        self.thread_receiver.try_recv().unwrap_or(Err(Error::NotYetConnected))
    }
}

type Client = websocket::sync::client::Client<Box<dyn websocket::sync::stream::NetworkStream + Send>>;

pub struct Connection {
    ws: Client
}

impl super::Connection for Connection {
    fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let msg = websocket::Message::binary(bytes);
        self.ws.send_message(&msg)?;
        Ok(())
    }

    fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>> {
        match self.ws.recv_message() {
            Ok(websocket::OwnedMessage::Binary(data)) => Ok(Some(data)),
            Ok(_) => Ok(None),
            Err(websocket::WebSocketError::IoError(io_err)) => {
                if io_err.kind() == std::io::ErrorKind::WouldBlock { Ok(None) }
                else { Err(io_err.into()) }
            }
            Err(other) => Err(other.into())
        }
    }
}

impl convert::From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self { Error::ConnectionError } // TODO
}

impl convert::From<websocket::WebSocketError> for Error {
    fn from(_: websocket::WebSocketError) -> Self { Error::ConnectionError } // TODO
}