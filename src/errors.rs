use std::sync::Arc;

use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::error;

/// Happy little general-purpose response error. This should be used for all the
/// errors returned by any route handler, because it's also responsible for
/// doing the proper formatting to show users.
#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("internal server error")]
    AxumExtensionRejection(#[from] axum::extract::rejection::ExtensionRejection),

    #[error("internal server error")]
    AxumHttpError(#[from] axum::http::Error),

    #[error("internal server error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("x-real-ip header empty or unreadable")]
    EmptyXRealIP(#[from] axum::http::header::ToStrError),

    #[error("x-real-ip header malformed")]
    InvalidXRealIP(#[from] std::net::AddrParseError),

    #[error("not found")]
    NotFoundError(),

    #[error("unauthorized")]
    TypedHeaderRejection(#[from] axum_extra::typed_header::TypedHeaderRejection),

    #[error("unauthorized")]
    Unauthorized(),
}

impl ResponseError {
    /// Little helper function that's only used to decide whether to log
    /// something or not. We want logging for most errors, but we don't really
    /// care about generic 404s.
    fn maybe_log(&self) {
        match self {
            Self::NotFoundError() => {}
            _ => {
                error!("response error: {:?}", self);
            }
        }
    }

    /// Maps a response error to a [StatusCode]. Only a few known cases are
    /// matched here, the default is [StatusCode::INTERNAL_SERVER_ERROR].
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized() | Self::TypedHeaderRejection(_) => StatusCode::UNAUTHORIZED,
            Self::EmptyXRealIP(_) | Self::InvalidXRealIP(_) => StatusCode::BAD_REQUEST,
            Self::NotFoundError() => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Renders the error as a plain text response. This is a fallback in case
    /// other methods fail, or if the client does not send an Accept header.
    fn plain_text_response(&self) -> Response {
        self.maybe_log();
        (
            self.status_code(),
            format!("{}: {}", self.status_code().as_u16(), self),
        )
            .into_response()
    }

    /// Renders the error as a JSON. The JSON has two fields: `code`, containing
    /// the HTTP error code, and `message`, which is a short string explaining
    /// what's wrong.
    fn json_response(&self) -> Response {
        self.maybe_log();
        let payload = json!({
            "code": self.status_code().as_u16(),
            "message": self.to_string()
        });

        (self.status_code(), Json(payload)).into_response()
    }

    /// Middleware function used to render content-type appropriate errors and
    /// designed for [axum::middleware::from_fn_with_state]. It extracts
    /// [ResponseError] from the [axum::Extension] if available, reads the
    /// Accept header, and decides which of the handlers to call. As a fallback,
    /// it presents the default response.
    pub async fn handle_error_middleware(
        req: Request,
        next: axum::middleware::Next,
    ) -> Result<Response, ResponseError> {
        let accept_type = req
            .headers()
            .get(axum::http::header::ACCEPT)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_owned());

        let response = next.run(req).await;

        if (response.status().is_client_error() || response.status().is_server_error())
            && let Some(error) = response.extensions().get::<Arc<ResponseError>>()
            && let Some(accept_type) = accept_type
        {
            let accept_type = accept_type.to_lowercase();
            if accept_type.contains("application/json") {
                return Ok(error.json_response());
            }
        }

        Ok(response)
    }
}

impl IntoResponse for ResponseError {
    /// This is sneaky. It returns the plain text response for an error, but it
    /// also adds [`Arc<Self>`] as an [axum::Extension] so we can extract that
    /// again inside [Self::handle_error_middleware].
    fn into_response(self) -> Response {
        let mut response = self.plain_text_response();
        response.extensions_mut().insert(Arc::new(self));
        response
    }
}
