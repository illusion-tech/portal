use futures::{SinkExt, StreamExt};
use warp::ws::{Message, WebSocket, Ws};
use warp::Filter;

use dashmap::DashMap;
use std::sync::Arc;
pub use portal_lib::*;

use tokio::net::TcpListener;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::stream::{SplitSink, SplitStream};
use lazy_static::lazy_static;

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

use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry;

use tracing::{error, info, Instrument};

lazy_static! {
    pub static ref CLI: cli::Cli = cli::Cli::parse();

    pub static ref CONNECTIONS: Connections = Connections::new();
    pub static ref ACTIVE_STREAMS: ActiveStreams = Arc::new(DashMap::new());
    pub static ref CONFIG: Config = match CLI.config {
        Some(ref config_path) => Config::load_from_file(config_path.to_str().unwrap()).unwrap(),
        None => Config::load_from_env()
    };

    // To disable all authentication:
    pub static ref AUTH_DB_SERVICE: crate::auth::NoAuth = crate::auth::NoAuth;
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

    tracing::info!("starting server!");

    control_server::spawn(([0, 0, 0, 0], CONFIG.control_port));
    info!("started portal server on 0.0.0.0:{}", CONFIG.control_port);

    network::spawn(([0, 0, 0, 0, 0, 0, 0, 0], CONFIG.internal_network_port));
    info!(
        "start network service on [::]:{}",
        CONFIG.internal_network_port
    );

    let listen_addr = format!("[::]:{}", CONFIG.remote_port);
    info!("listening on: {}", &listen_addr);
    info!("portal server with hostname: {}", &CONFIG.portal_host);

    // create our accept any server
    let listener = TcpListener::bind(listen_addr)
        .await
        .expect("failed to bind");

    loop {
        let socket = match listener.accept().await {
            Ok((socket, _)) => socket,
            _ => {
                error!("failed to accept socket");
                continue;
            }
        };

        tokio::spawn(
            async move {
                remote::accept_connection(socket).await;
            }
            .instrument(observability::remote_trace("remote_connect")),
        );
    }
}
