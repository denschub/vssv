use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{errors::ResponseError, ServerState};

/// Liveness probe endpoint. Always responds with `204 No Content` as soon as
/// the server is up.
#[axum::debug_handler]
pub async fn get_livez() -> impl IntoResponse {
    (StatusCode::NO_CONTENT, Body::empty())
}

/// Readiness probe. Respeonds with `204 No Content` if the server is up and the
/// database can respond to queries. Responds with a 500 otherwise.
#[axum::debug_handler]
pub async fn get_readyz(State(state): State<ServerState>) -> Result<Response, ResponseError> {
    let _ = sqlx::query!(
        r#"select
        now() as test_timestamp,
        uuid_generate_v4() as test_uuid,
        encode(gen_random_bytes(1), 'hex') as random_byte;"#
    )
    .fetch_all(&state.database)
    .await?;

    Ok((StatusCode::NO_CONTENT, Body::empty()).into_response())
}
