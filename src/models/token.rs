use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgExecutor};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{errors::ResponseError, ServerState};

/// An access token stored in the database.
#[derive(Debug)]
pub struct Token {
    pub uuid: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub token: String,
    pub superuser: bool,
}

impl Token {
    /// Tries to find a Token from the database based on its token value. If
    /// nothing is found, it will result with None().
    pub async fn try_query_with_token<'e>(
        db: impl PgExecutor<'e>,
        token: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "select uuid, expires_at, token, superuser from tokens where token = $1",
            token
        )
        .fetch_optional(db)
        .await
    }

    /// Little helper that checks if a token is expired. If the token has no
    /// expiration date, it will always return `false`.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            None => false,
            Some(expiry) => expiry < Utc::now(),
        }
    }

    /// Updates the used_at timestamp in the database. This should be called
    /// early in the chain, as soon as the token is validated to be existing -
    /// even if it's expired. This allows tracking expired tokens that are still
    /// used.
    pub async fn update_used_timestamp<'e>(
        &mut self,
        db: impl PgExecutor<'e>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "update tokens set used_at = $1 where uuid = $2",
            Utc::now(),
            self.uuid
        )
        .execute(db)
        .await
    }

    /// Checks if this Token can read a given secret (looked up by the secret's
    /// UUID). This always returns `true` it the token is a superuser token.
    /// Otherwise, it checks that a token_permission with can_read=true exists.
    pub async fn can_read_secret<'e>(
        &self,
        db: impl PgExecutor<'e>,
        secret_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        if self.superuser {
            return Ok(true);
        }

        Ok(sqlx::query!(
            r#"select token from token_permissions where token = $1 and secret = $2 and can_read = true"#,
            self.uuid,
            secret_uuid
        ).fetch_optional(db).await?.is_some())
    }

    /// Checks if this Token can write a given secret (looked up by the secret's
    /// UUID). This always returns `true` it the token is a superuser token.
    /// Otherwise, it checks that a token_permission with can_write=true exists.
    pub async fn can_write_secret<'e>(
        &self,
        db: impl PgExecutor<'e>,
        secret_uuid: Uuid,
    ) -> Result<bool, sqlx::Error> {
        if self.superuser {
            return Ok(true);
        }

        Ok(sqlx::query!(
            r#"select token from token_permissions where token = $1 and secret = $2 and can_write = true"#,
            self.uuid,
            secret_uuid
        ).fetch_optional(db).await?.is_some())
    }
}

#[derive(Debug)]
pub struct ExtractValidToken(pub Token);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractValidToken
where
    ServerState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ResponseError;

    /// Happy little [FromRequestParts] implementation that always extracts a valid
    /// token. "Valid" means here: it exists, and it's not expired. This
    /// extractor will also set the token's used_at timestamp, and it does that
    /// even when the token is expired, so there is a way to track the usage of
    /// expired tokens.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = ServerState::from_ref(state);

        let token_header = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await?
            .0;

        let token = Token::try_query_with_token(&app_state.database, token_header.token()).await?;
        let Some(mut token) = token else {
            info!("use of invalid token=`{}`", token_header.token());
            return Err(Self::Rejection::Unauthorized());
        };

        let _ = token.update_used_timestamp(&app_state.database).await?;

        if token.is_expired() {
            warn!("use of expired token=`{}`", token.uuid);
            return Err(Self::Rejection::Unauthorized());
        }

        Ok(Self(token))
    }
}
