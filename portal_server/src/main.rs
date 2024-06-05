use futures::{SinkExt, StreamExt};
use warp::ws::{Message, WebSocket, Ws};
use warp::Filter;

use dashmap::DashMap;
pub use portal_lib::*;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use tokio::net::{TcpListener, TcpStream};

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::{SplitSink, SplitStream};

mod connected_clients;
use self::connected_clients::*;
mod active_stream;
use self::active_stream::*;

mod auth;
pub use self::auth::client_auth;

// pub use self::auth_db::AuthDbService;

mod control_server;
mod remote;

mod config;
pub use self::config::Config;
mod network;

mod observability;

mod cli;
use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tokio::time::sleep;
use cli::Cli;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry;

use tracing::{error, info, Instrument};
use tracing::log::{debug, warn};

static CLI: OnceLock<Cli> = OnceLock::new();
static CONNECTIONS: OnceLock<Connections> = OnceLock::new();
static ACTIVE_STREAMS: OnceLock<ActiveStreams> = OnceLock::new();
static CONFIG: OnceLock<Config> = OnceLock::new();
static AUTH_DB_SERVICE: OnceLock<crate::auth::NoAuth> = OnceLock::new();


pub fn get_cli() -> &'static Cli {
    CLI.get_or_init(Cli::parse)
}

pub fn get_connections() -> &'static Connections {
    CONNECTIONS.get_or_init(Connections::new)
}

pub fn get_active_streams() -> &'static ActiveStreams {
    ACTIVE_STREAMS.get_or_init(|| Arc::new(DashMap::new()))
}

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| match get_cli().config {
        Some(ref config_path) => Config::load_from_file(config_path.to_str().unwrap()).unwrap(),
        None => Config::load_from_env(),
    })
}

pub fn get_auth_db_service() -> &'static crate::auth::NoAuth {
    AUTH_DB_SERVICE.get_or_init(|| crate::auth::NoAuth)
}

#[tokio::main]
async fn main() {
    // if let Some(config_path) = &CLI.config {
    //     println!("Value for config: {}", config_path.display());
    // };
    // setup observability
    let subscriber = registry::Registry::default()
        .with(LevelFilter::DEBUG)
        .with(tracing_subscriber::fmt::Layer::default());
    tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");

    info!("starting server!");
    let config = get_config();

    network::spawn(([0, 0, 0, 0, 0, 0, 0, 0], config.internal_network_port));
    info!(
        "start network service on [::]:{}",
        config.internal_network_port
    );
    let listener = TcpListener::bind(format!("[::]:{}", config.remote_port))
        .await
        .expect("failed to bind");

    control_server::spawn(([0, 0, 0, 0], config.control_port));
    info!(
        "started portal control server on 0.0.0.0:{}",
        config.control_port
    );
    // create our accept any server
    // let listener = TcpListener::bind(listen_addr)
    //     .await
    //     .expect("failed to bind");
    loop {
        tokio::select! {

            accept_result = listener.accept() => {
                let mut new_socket = match accept_result {
                    Ok((socket, _)) => socket,
                    Err(e) => {
                        error!("failed to accept socket: {:?}", e);
                        continue;
                    }
                };
                let peer_addr = new_socket.peer_addr().unwrap();
                info!("accepted connection from: {}", peer_addr);

                tokio::spawn(
                    async move {
                        remote::accept_connection(new_socket).await;
                    }.instrument(observability::remote_trace("remote_connect")),
                );
            },
        }
    }
}