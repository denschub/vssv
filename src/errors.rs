use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("internal server error")]
    AxumExtensionRejection(#[from] axum::extract::rejection::ExtensionRejection),

    #[error("internal server error")]
    AxumHttpError(#[from] axum::http::Error),

    #[error("internal server error")]
    DbError(#[from] sqlx::Error),

    #[error("x-real-ip header empty or unreadable")]
    EmptyXRealIP(#[from] axum::http::header::ToStrError),

    #[error("x-real-ip header malformed")]
    InvalidXRealIP(#[from] std::net::AddrParseError),

    #[error("nothing found")]
    NoneFound(),

    #[error("unauthorized")]
    TypedHeaderRejection(#[from] axum_extra::typed_header::TypedHeaderRejection),

    #[error("unauthorized")]
    Unauthorized(),
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        if let ResponseError::DbError(db_err) = &self {
            error!("unexpected DB error: {}", db_err);
        }

        let status_code = match self {
            ResponseError::Unauthorized() | ResponseError::TypedHeaderRejection(_) => {
                StatusCode::UNAUTHORIZED
            }
            ResponseError::EmptyXRealIP(_) | ResponseError::InvalidXRealIP(_) => {
                StatusCode::BAD_REQUEST
            }
            ResponseError::NoneFound() => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder()
            .status(status_code)
            .body(Body::from(self.to_string()))
            .expect("Response Builder with known setup should not fail")
            .into_response()
    }
}
