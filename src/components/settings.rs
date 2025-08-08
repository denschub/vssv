use std::net::SocketAddr;

use sqlx::postgres::PgConnectOptions;

/// Specifies the log's output format
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogFormat {
    Text,
    TextColor,
    Json,
}

/// Specifies how much log output the app generates
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn tracing_level(&self) -> tracing::Level {
        use tracing::Level;

        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

#[derive(Clone, Debug, clap::Parser)]
#[clap(about, version, propagate_version = true)]
pub struct Settings {
    /// The database URL to connect to. Needs to be a valid PostgreSQL
    /// connection URL, like `postgres://postgres@127.0.0.1/vssv`
    #[clap(long, env = "DATABASE_URL")]
    pub database_url: PgConnectOptions,

    /// The Socket address the server should listen on
    #[clap(long, env = "LISTEN", default_value = "[::1]:8081")]
    pub listen: SocketAddr,

    /// Defines how the log output will be formatted
    #[clap(value_enum, long, env = "LOG_FORMAT", default_value_t = LogFormat::TextColor)]
    pub log_format: LogFormat,

    /// Defines how noisy the server should be
    #[clap(value_enum, long, env = "LOG_LEVEL", default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// Limits the number of threads used - defaults to the number of CPU cores
    #[clap(long, env = "THREADS")]
    pub threads: Option<usize>,

    /// If set, the X-Real-IP header will be used for the audit log IPs
    #[clap(long, env = "USE_X_REAL_IP")]
    pub use_x_real_ip: bool,
}
