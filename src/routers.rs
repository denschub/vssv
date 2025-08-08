mod app_meta;
mod secrets;

use axum::{Router, middleware};

use crate::{AppState, errors::ResponseError};

/// Builds the main router.
/// This should collect all the routes from all over the app, and return a
/// full router for use in the app.
pub fn build_main_router(state: AppState) -> Router {
    let error_handling_layer =
        middleware::from_fn_with_state(state.clone(), ResponseError::handle_error_middleware);

    Router::new()
        .merge(app_meta::build())
        .merge(secrets::build())
        .layer(error_handling_layer)
        .fallback(fallback_handler)
        .with_state(state)
}

#[axum::debug_handler]
async fn fallback_handler() -> Result<(), ResponseError> {
    Err(ResponseError::NotFoundError())
}
