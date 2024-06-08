use sqlx::PgPool;

pub mod errors;
pub mod models;
pub mod routes;

/// Holds the web server's state.
#[derive(Clone)]
pub struct ServerState {
    pub database: PgPool,
}
