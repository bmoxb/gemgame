mod handling;
mod networking;
mod world;

use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex}
};

use shared::WEBSOCKET_CONNECTION_PORT;
use structopt::StructOpt;
use tokio::net::TcpListener;
use world::World;

#[tokio::main]
async fn main() {
    // Command-line arguments:

    let mut options = Options::from_args();
    if options.port == 0 {
        options.port = WEBSOCKET_CONNECTION_PORT;
    }

    // Logger initialisation:

    let mut logger = flexi_logger::Logger::with_str(options.log_level)
        .log_target(flexi_logger::LogTarget::StdOut)
        .format_for_stdout(flexi_logger::colored_detailed_format);

    if options.log_to_file {
        logger = logger
            .log_target(flexi_logger::LogTarget::File)
            .format_for_files(flexi_logger::detailed_format)
            .duplicate_to_stdout(flexi_logger::Duplicate::All)
            .rotate(
                flexi_logger::Criterion::Age(flexi_logger::Age::Day),
                flexi_logger::Naming::Timestamps,
                flexi_logger::Cleanup::KeepLogFiles(3)
            );
    }
    logger.start().expect("Failed to initialise logger");

    // Prepare data structures that are to be shared between threads:

    let connections: Shared<ConnectionRecords> = Arc::new(Mutex::new(HashMap::new()));

    let world: Shared<World> =
        Arc::new(Mutex::new(World::new(options.world_directory.clone()).expect("Failed to create game world")));

    // Bind socket and handle connections:

    let host_address = format!("127.0.0.1:{}", options.port);

    match TcpListener::bind(&host_address).await {
        Ok(listener) => {
            log::info!("Created TCP/IP listener bound to address: {}", host_address);

            while let Ok((stream, addr)) = listener.accept().await {
                log::info!("Incoming connection from: {}", addr);

                tokio::spawn(handling::handle_connection(stream, addr, Arc::clone(&connections), Arc::clone(&world)));
            }
        }

        Err(e) => {
            log::error!("Failed to create TCP/IP listener at '{}' due to error - {}", host_address, e);
        }
    }
}

type Shared<T> = Arc<Mutex<T>>;

type ConnectionRecords = HashMap<SocketAddr, ConnectionRecord>;

pub struct ConnectionRecord {
    current_map_key: String
}

// TODO: When Clap version 3 is stable, use that instead?
/// Server application for not-yet-named web MMO roguelike.
#[derive(StructOpt, Debug)]
#[structopt(name = "MMO Server")]
struct Options {
    /// The port on which listen for incoming connections.
    #[structopt(short, long, default_value = "0")]
    port: u16,

    /// Directory containing game world data.
    #[structopt(long, default_value = "world/", parse(from_os_str))]
    world_directory: PathBuf,

    /// Specify the logging level (trace, debug, info, warn, error).
    #[structopt(short, long, default_value = "info")]
    log_level: String,

    /// Specifiy whether or not log messages should be written to a file in addition to stdout.
    #[structopt(long)]
    log_to_file: bool
}
