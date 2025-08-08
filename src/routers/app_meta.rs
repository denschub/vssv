use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::json;

use crate::{AppState, errors::ResponseError};

/// Builds the fallback router.
pub fn build() -> Router<AppState> {
    Router::new()
        .route("/livez", get(livez_handler))
        .route("/readyz", get(readyz_handler))
        .route("/versionz", get(versionz_handler))
}

/// `/livez` handler that always returns a 200
#[axum::debug_handler]
#[tracing::instrument]
async fn livez_handler() -> impl IntoResponse {
    StatusCode::OK
}

/// `/readyz` handler that returns a 200 if everything is good, or a 500
/// otherwise.
#[axum::debug_handler]
#[tracing::instrument(skip(app_state))]
async fn readyz_handler(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ResponseError> {
    let _ = sqlx::query!(
        r#"select
        now() as test_time,
        uuid_generate_v4() as test_uuid"#
    )
    .fetch_all(&app_state.database)
    .await?;

    Ok(StatusCode::OK)
}

/// `/versionz` handler that resturns a JSON object containing this app's
/// version number, and some commit info.
#[axum::debug_handler]
#[tracing::instrument]
async fn versionz_handler() -> impl IntoResponse {
    Json(json!({
        "git": env!("VERGEN_GIT_SHA"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
