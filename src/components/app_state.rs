use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AppState {
    pub database: sqlx::PgPool,
    pub settings: Arc<super::settings::Settings>,
}
