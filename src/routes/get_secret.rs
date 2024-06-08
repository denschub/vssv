use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, State},
    response::{IntoResponse, Response},
};
use tracing::warn;
use uuid::Uuid;

use crate::{
    errors::ResponseError,
    models::{audit_log_entry::AuditLogAction, token::ExtractValidToken, AuditLogEntry, Secret},
    ServerState,
};

/// Endpoint that allows reading secrets. All requests require a valid token. In
/// additoin, all requests are gated behind the can_read token permissions. It
/// always returns a 401 if the token is valid but can't read a secret, no
/// matter if the secret actually exists or not.
#[axum::debug_handler]
pub async fn get_secret(
    State(state): State<ServerState>,
    Path(uuid): Path<Uuid>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
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
        client_addr.ip(),
        AuditLogAction::SecretRead,
        token.uuid,
        secret.uuid,
    )
    .await?;

    Ok(secret.into_response())
}
