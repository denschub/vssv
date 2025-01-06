use std::net::SocketAddr;

use sqlx::postgres::PgConnectOptions;

#[derive(Debug, clap::Parser)]
#[clap(about, version, propagate_version = true)]
pub struct Settings {
    /// The database URL to connect to. Needs to be a valid PostgreSQL
    /// connection URL, like `postgres://postgres@127.0.0.1/vssv`
    #[clap(long, short, env = "DATABASE_URL")]
    pub database_url: PgConnectOptions,

    /// The Socket Address the server should listen on
    #[clap(long, short, env = "LISTEN_ADDR", default_value = "[::1]:3000")]
    pub listen_addr: SocketAddr,

    /// Limits the number of threads used - defaults to the number of CPU cores
    #[clap(long, env = "THREADS")]
    pub threads: Option<usize>,

    /// If set, the X-Real-IP header will be used for the audit log IPs
    #[clap(long, env = "USE_X_REAL_IP")]
    pub use_x_real_ip: bool,
}
