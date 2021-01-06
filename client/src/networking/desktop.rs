use std::{convert, net, sync::mpsc, thread};

use tungstenite as ws2;

use super::{Error, Result};

pub struct PendingConnection {
    thread_receiver: mpsc::Receiver<Result<Connection>>
}

impl super::PendingConnectionTrait<Connection> for PendingConnection {
    fn new(full_url: String) -> Self {
        let (thread_sender, thread_receiver) = mpsc::channel();

        thread::spawn(move || {
            let result = match ws2::connect(&full_url) {
                Ok((ws, _)) => {
                    log::debug!("Established WebSocket connection to URL: '{}'", full_url);

                    let tcp_socket = match ws.get_ref() {
                        ws2::stream::Stream::Plain(tcp) => tcp,
                        ws2::stream::Stream::Tls(tls) => tls.get_ref()
                    };
                    tcp_socket.set_nonblocking(true).expect("Failed to transition to non-blocking mode");
                    log::debug!("Underlying TCP/IP socket made to enter non-blocking mode");

                    Ok(Connection { ws })
                }

                Err(e) => {
                    log::error!("Failed to establish WebSocket connection: {}", e);
                    Err(e.into())
                }
            };

            thread_sender.send(result).unwrap();
        });

        PendingConnection { thread_receiver }
    }

    fn ready(&self) -> Result<Option<Connection>> {
        match self.thread_receiver.try_recv() {
            Ok(Ok(conn)) => Ok(Some(conn)),
            Ok(Err(e)) => Err(e),
            Err(_) => Ok(None)
        }
    }
}

/// WebSocket connection relying on the `tungstenite` library's implementation of the protocol.
pub struct Connection {
    ws: ws2::WebSocket<ws2::client::AutoStream>
}

impl super::ConnectionTrait for Connection {
    fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let msg = ws2::Message::binary(bytes);
        self.ws.write_message(msg)?;
        Ok(())
    }

    fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>> {
        match self.ws.read_message() {
            Ok(msg) => match msg {
                // Return binary message:
                ws2::Message::Binary(data) => Ok(Some(data)),

                // Complete closing handshake and indicate to the caller that the connection is now closed:
                ws2::Message::Close(_) => {
                    log::debug!("Performing closing handshake");

                    let _ = self.ws.close(None).and_then(|_| self.ws.write_pending());
                    Err(Error::ConnectionClosed)
                }

                // Any other message type is ignored:
                _ => Ok(None)
            },

            Err(ws2::Error::Io(io_error)) => {
                if io_error.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(None)
                }
                else {
                    Err(io_error.into())
                }
            }

            Err(other_error) => Err(other_error.into())
        }
    }
}

impl convert::From<ws2::Error> for Error {
    fn from(e: ws2::Error) -> Self { Error::ConnectionError(Box::new(e)) }
}

impl convert::From<ws2::HandshakeError<ws2::ClientHandshake<net::TcpStream>>> for Error {
    fn from(e: ws2::HandshakeError<ws2::ClientHandshake<net::TcpStream>>) -> Self {
        Error::ConnectionError(Box::new(e))
    }
}
