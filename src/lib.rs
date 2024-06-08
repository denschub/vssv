use std::net::SocketAddr;

use sqlx::{postgres::PgConnectOptions, PgPool};

pub mod errors;
pub mod models;
pub mod routes;

#[derive(Debug, clap::Parser)]
#[clap(about, version, propagate_version = true)]
pub struct Cli {
    /// The database URL to connect to. Needs to be a valid PostgreSQL
    /// connection URL, like `postgres://postgres@127.0.0.1/vssv`
    #[clap(long, short, env = "DATABASE_URL")]
    pub database_url: PgConnectOptions,

    /// The Socket Address the server should listen on
    #[clap(long, short, env = "LISTEN_ADDR", default_value = "[::1]:3000")]
    pub listen_addr: SocketAddr,
}

/// Holds the web server's state.
#[derive(Clone)]
pub struct ServerState {
    pub database: PgPool,
}
