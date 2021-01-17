mod handling;
mod id;
mod networking;
mod world;

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex}
};

use shared::WEBSOCKET_CONNECTION_PORT;
use structopt::StructOpt;
use tokio::{net::TcpListener, sync::broadcast};
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

    // Bind socket and handle connections:

    let host_address = format!("127.0.0.1:{}", options.port);

    let listener = TcpListener::bind(&host_address).await.expect("Failed to create TCP/IP listener");
    log::info!("Created TCP/IP listener bound to address: {}", host_address);

    // Load/create game world that is to be shared between threads:

    let world: Shared<World> =
        Arc::new(Mutex::new(World::new(options.world_directory.clone()).expect("Failed to load/create game world")));
    log::info!("Loaded/created game world from directory: {}", options.world_directory.display());

    // Connect to database:

    let _ = fs::OpenOptions::new().append(true).create(true).open(&options.database_file); // Create file if not exists.

    let connection_string = format!("sqlite://{}", &options.database_file);

    let db_pool_options = sqlx::sqlite::SqlitePoolOptions::new().max_connections(options.max_database_connections);
    let db_pool = db_pool_options.connect(&connection_string).await.expect("Failed to connect to database");

    log::info!(
        "Created connection pool with maximum of {} simultaneous connections to database: {}",
        options.max_database_connections,
        connection_string
    );

    let create_table_query = sqlx::query(
        "CREATE TABLE IF NOT EXISTS client_entities (
            client_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL UNIQUE,
            current_map_id TEXT NOT NULL,
            tile_x INTEGER NOT NULL,
            tile_y INTEGER NOT NULL
        )"
    );
    create_table_query.execute(&db_pool).await.expect("Failed to create required table in database");

    log::info!("Prepared necessary database table");

    // Create multi-producer, multi-consumer channel so that each task may notify every other task of changes
    // made to the game world:

    let (world_changes_sender, mut world_changes_receiver) = broadcast::channel(5);

    // The 'select' macro below means that connections will be continuously listened for unless Ctrl-C is
    // pressed and the loop is exited. Messages on the world modifcation channel are also listened for and
    // immediately discarded. This is done as the main task must maintain access to the channel in order to
    // clone and pass it to new connection tasks while also not blocking the broadcasted message queue.

    log::info!("Listening for incoming TCP/IP connections...");

    while let Some(src) = tokio::select!(
        res = listener.accept() => Some(ReceivedOn::NetworkConnection(res)),
        res = world_changes_receiver.recv() => Some(ReceivedOn::TokioBroadcast(res)),
        _ = tokio::signal::ctrl_c() => None
    ) {
        match src {
            ReceivedOn::NetworkConnection(res) => {
                let (stream, addr) = res.unwrap();

                log::info!("Incoming connection from: {}", addr);

                tokio::spawn(handling::handle_connection(
                    stream,
                    addr,
                    Arc::clone(&world),
                    db_pool.clone(),
                    world_changes_sender.subscribe()
                ));
            }

            ReceivedOn::TokioBroadcast(_) => {} // Discard the broadcasted world modification message.
        }
    }

    log::info!("No longer listening for connections");

    // TODO: Save game world before closing the program.
}

enum ReceivedOn<T> {
    NetworkConnection(T),
    TokioBroadcast(Result<world::Modification, broadcast::error::RecvError>)
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

    /// Directory containing game world data.
    #[structopt(long, default_value = "world/", parse(from_os_str))]
    world_directory: PathBuf,

    /// The database file in which to store client/player data.
    #[structopt(long, default_value = "clients.db")]
    database_file: String,

    /// Specify the maximum number of connections that the database connection pool is able to have open
    /// simultaneously.
    #[structopt(long, default_value = "25")]
    max_database_connections: u32,

    /// Specify the logging level (trace, debug, info, warn, error).
    #[structopt(short, long, default_value = "info")]
    log_level: String,

    /// Specifiy whether or not log messages should be written to a file in addition to stdout.
    #[structopt(long)]
    log_to_file: bool
}
