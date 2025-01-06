use sqlx::PgPool;

pub mod errors;
pub mod models;
pub mod routes;
pub mod settings;

/// Holds the web server's state.
#[derive(Clone)]
pub struct ServerState {
    pub database: PgPool,
    pub use_x_real_ip: bool,
}
