mod world;

use std::{
    path::PathBuf,
    net::SocketAddr,
    sync::{ Arc, Mutex }
};

use tokio::net::{ TcpListener, TcpStream };

use structopt::StructOpt;

#[tokio::main]
async fn main() {
    // Command-line arguments:

    let options = Options::from_args();

    // Logger initialisation:

    let mut logger = flexi_logger::Logger::with_str(options.log_level)
        .log_target(flexi_logger::LogTarget::StdOut)
        .format_for_stdout(flexi_logger::colored_detailed_format);

    if options.log_to_file {
        logger = logger.log_target(flexi_logger::LogTarget::File)
                       .format_for_files(flexi_logger::detailed_format)
                       .duplicate_to_stdout(flexi_logger::Duplicate::All)
                       .rotate(
                            flexi_logger::Criterion::Age(flexi_logger::Age::Day),
                            flexi_logger::Naming::Timestamps,
                            flexi_logger::Cleanup::KeepLogFiles(3)
                       );
    }
    logger.start().expect("Failed to initialise logger");

    // Bind socket and handle connections:

    let host_address = format!("127.0.0.1:{}", options.port);

    let shared = Arc::new(Mutex::new(Shared {
        game_world: world::World::load_or_new(options.world_directory.clone())
    }));

    match TcpListener::bind(&host_address).await {
        Ok(listener) => {
            log::info!("Created TCP/IP listener bound to address: {}", host_address);

            while let Ok((stream, addr)) = listener.accept().await {
                log::info!("Incoming connection from: {}", addr);

                tokio::spawn(handle_connection(stream, addr, shared.clone()));
            }
        }

        Err(e) => {
            log::error!("Failed to create TCP/IP listener at '{}' due to error - {}",
                        host_address, e);
        }
    }
}

// TODO: When Clap version 3 is stable, use that instead?
/// Server application for not-yet-named web MMO roguelike.
#[derive(StructOpt, Debug)]
#[structopt(name = "MMO Server")]
struct Options {
    /// The port on which listen for incoming connections.
    #[structopt(short, long, default_value = "8000")]
    port: usize,

    /// Directory containing game world data.
    #[structopt(long, default_value = "world/", parse(from_os_str))]
    world_directory: PathBuf,

    /// Specify the logging level (trace, debug, info, warn, error).
    #[structopt(short, long, default_value = "info")]
    log_level: String,

    /// Specifiy whether or not log messages should be written to a file in
    /// addition to stdout.
    #[structopt(long)]
    log_to_file: bool
}

/// Data shared between all connection threads.
struct Shared {
    game_world: world::World
}

/// Handle a connection with an individual client. This function is called
/// concurrently as a Tokio task.
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