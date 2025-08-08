use axum::{
    Router,
    body::{Body, Bytes},
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use tracing::warn;
use uuid::Uuid;

use crate::{
    components::app_state::AppState,
    entities::{AuditLogAction, AuditLogEntry, ExtractClientAddr, ExtractValidToken, Secret},
    errors::ResponseError,
};

pub fn build() -> Router<AppState> {
    Router::new()
        .route("/secret/{uuid}", get(get_secret))
        .route("/secret/{uuid}/contents", post(post_secret_contents))
}

/// Endpoint that allows reading secrets. All requests require a valid token. In
/// addition, all requests are gated behind the can_read token permissions. It
/// always returns a 401 if the token is valid but can't read a secret, no
/// matter if the secret actually exists or not.
#[axum::debug_handler]
pub async fn get_secret(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    ExtractClientAddr(client_addr): ExtractClientAddr,
    ExtractValidToken(token): ExtractValidToken,
) -> Result<Response, ResponseError> {
    if !token.can_read_secret(&state.database, uuid).await? {
        warn!(
            "token=`{}` not allowed to read secret=`{}`",
            token.uuid, uuid
        );
        return Err(ResponseError::Unauthorized());
    }

    let secret = Secret::find(&state.database, uuid).await?;
    let _ = AuditLogEntry::log_action(
        &state.database,
        client_addr.ip,
        AuditLogAction::SecretRead,
        token.uuid,
        secret.uuid,
    )
    .await?;

    Ok(secret.into_response())
}

/// Endpoint that allows updating a secret's contents. All requests require a
/// valid token. In additoin, all requests are gated behind the can_write token
/// permissions. It always returns a 401 if the token is valid but can't write a
/// secret, no matter if the secret actually exists or not.
#[axum::debug_handler]
pub async fn post_secret_contents(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    ExtractClientAddr(client_addr): ExtractClientAddr,
    ExtractValidToken(token): ExtractValidToken,
    body: Bytes,
) -> Result<Response, ResponseError> {
    if !token.can_write_secret(&state.database, uuid).await? {
        warn!(
            "token=`{}` not allowed to write secret=`{}`",
            token.uuid, uuid
        );
        return Err(ResponseError::Unauthorized());
    }

    let mut secret = Secret::find(&state.database, uuid).await?;
    let _ = AuditLogEntry::log_action(
        &state.database,
        client_addr.ip,
        AuditLogAction::SecretWrite,
        token.uuid,
        secret.uuid,
    )
    .await?;

    let _ = secret
        .update_contents(&state.database, body.to_vec())
        .await?;

    Ok((StatusCode::NO_CONTENT, Body::empty()).into_response())
}
