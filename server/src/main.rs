mod handling;
mod id;
mod maps;
mod networking;

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex}
};

use maps::ServerMap;
use structopt::StructOpt;
use tokio::{net::TcpListener, sync::broadcast};

pub const WEBSOCKET_PORT: u16 = 5678;

#[tokio::main]
async fn main() {
    // Command-line arguments:

    let mut options = Options::from_args();
    if options.port == 0 {
        options.port = WEBSOCKET_PORT;
    }

    // Logger initialisation:

    let log_level = {
        if options.log_debug {
            flexi_logger::Level::Debug
        }
        else if options.log_trace {
            flexi_logger::Level::Trace
        }
        else {
            flexi_logger::Level::Info
        }
    }
    .to_level_filter();

    let mut log_spec_builder = flexi_logger::LogSpecBuilder::new();
    log_spec_builder.default(log_level);

    for module in &["sqlx", "tungstenite", "tokio_tungstenite", "mio"] {
        log_spec_builder.module(module, flexi_logger::LevelFilter::Warn);
    }

    let log_spec = log_spec_builder.finalize();

    let mut logger = flexi_logger::Logger::with(log_spec)
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

    // Bind socket and handle connections:

    let host_address = format!("0.0.0.0:{}", options.port);

    let listener = TcpListener::bind(&host_address).await.expect("Failed to create TCP/IP listener");
    log::info!("Created TCP/IP listener bound to address: {}", host_address);

    // Load/create game map that is to be shared between threads:

    let contained_map = ServerMap::try_load(options.map_directory.clone()).await;
    let map: Shared<ServerMap> = Arc::new(Mutex::new(contained_map));
    log::info!("Loaded/created game map from directory: {}", options.map_directory.display());

    // Connect to database:

    let db_pool_options = sqlx::any::AnyPoolOptions::new().max_connections(options.max_database_connections);
    let db_pool =
        db_pool_options.connect(&options.database_connection_string).await.expect("Failed to connect to database");

    log::info!(
        "Created connection pool with maximum of {} simultaneous connections to database: {}",
        options.max_database_connections,
        options.database_connection_string
    );

    let create_table_query = sqlx::query(
        "CREATE TABLE IF NOT EXISTS client_entities (
            client_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL UNIQUE,
            tile_x INTEGER NOT NULL,
            tile_y INTEGER NOT NULL,
            hair_style BYTEA NOT NULL,
            clothing_colour BYTEA NOT NULL,
            skin_colour BYTEA NOT NULL,
            hair_colour BYTEA NOT NULL,
            has_running_shoes BOOLEAN NOT NULL
        )"
    );
    create_table_query.execute(&db_pool).await.expect("Failed to create required table in database");

    log::info!("Prepared necessary database table");

    // Create multi-producer, multi-consumer channel so that each task may notify every other task of changes
    // made to the game world:

    let (map_changes_sender, mut map_changes_receiver) = broadcast::channel(5);

    log::info!("Listening for incoming TCP/IP connections...");

    loop {
        // Connections will be continuously listened for unless Ctrl-C is pressed and the loop is exited. Messages on
        // the world modifcation channel are also listened for and immediately discarded. This is done as the main task
        // must maintain access to the channel in order to clone and pass it to new connection tasks while also not
        // blocking the broadcasted message queue.
        tokio::select!(
            res = listener.accept() => {
                let (stream, address) = res.unwrap();

                log::info!("Incoming connection from: {}", address);

                tokio::spawn(handling::handle_connection(
                    stream,
                    address,
                    Arc::clone(&map),
                    db_pool.clone(),
                    map_changes_sender.clone(),
                    map_changes_sender.subscribe()
                ));
            }
            _ = map_changes_receiver.recv() => {} // Discard the broadcasted world modification message.
            _ = tokio::signal::ctrl_c() => break // Break on Ctrl-C.
        );
    }

    log::info!("No longer listening for connections");

    let contained_map = Arc::try_unwrap(map).ok().unwrap().into_inner().unwrap();
    if let Err(e) = contained_map.save_all().await {
        log::error!("Failed to save game map before exiting due to error: {}", e);
    }
}

type Shared<T> = Arc<Mutex<T>>;

// TODO: When Clap version 3 is stable, use that instead?
/// Server application for not-yet-named web MMO roguelike.
#[derive(StructOpt, Debug)]
#[structopt(name = "MMO Server")]
struct Options {
    /// The port on which listen for incoming connections.
    #[structopt(short, long, default_value = "0")]
    port: u16,

    /// Directory containing game map data.
    #[structopt(long, default_value = "map/", parse(from_os_str))]
    map_directory: PathBuf,

    /// Specify how to connect to the database.
    #[structopt(long, default_value = "postgres://db/gemgame")]
    database_connection_string: String,

    /// Specify the maximum number of connections that the database connection pool is able to have open
    /// simultaneously.
    #[structopt(long, default_value = "25")]
    max_database_connections: u32,

    /// Display all debugging logger messages.
    #[structopt(long, conflicts_with = "log-trace")]
    log_debug: bool,

    /// Display all tracing and debugging logger messages.
    #[structopt(long)]
    log_trace: bool,

    /// Specifiy whether or not log messages should be written to a file in addition to stdout.
    #[structopt(long)]
    log_to_file: bool
}
