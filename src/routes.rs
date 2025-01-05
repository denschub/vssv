mod get_secret;
mod health;
mod post_secret_contents;

use axum::{
    routing::{get, post},
    Router,
};

use get_secret::get_secret;
use health::{get_livez, get_readyz};
use post_secret_contents::post_secret_contents;

use crate::ServerState;

pub fn build_router(state: ServerState) -> Router {
    Router::new()
        .route("/livez", get(get_livez))
        .route("/readyz", get(get_readyz))
        .route("/secret/{uuid}", get(get_secret))
        .route("/secret/{uuid}/contents", post(post_secret_contents))
        .with_state(state)
}
