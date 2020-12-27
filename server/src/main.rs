mod maps;

use std::{
    net::SocketAddr,
    sync::{ Arc, Mutex }
};

use tokio::net::{ TcpListener, TcpStream };

const PORT: usize = 8080;

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let addr = format!("127.0.0.1:{}", PORT);

    let shared = Arc::new(Mutex::new(Shared {}));

    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            log::info!("Created TCP/IP listener bound to address: {}", addr);

            while let Ok((stream, addr)) = listener.accept().await {
                log::info!("Incoming connection from: {}", addr);

                tokio::spawn(handle_connection(stream, addr, shared.clone()));
            }
        }

        Err(e) => {
            log::error!("Failed to create TCP/IP listener due to error: {}", e);
        }
    }
}

/// Data shared between all connection threads.
struct Shared {}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, shared: Arc<Mutex<Shared>>) {
    match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => {
            log::debug!("Performed WebSocket handshake successfully with: {}", addr);

            // ...

            log::info!("Client disconnected: {}", addr);
        }

        Err(e) => {
            log::error!("Failed to perform WebSocket handshake with '{}' - {}", addr, e);
        }
    }
}