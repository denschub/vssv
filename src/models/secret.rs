use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::{postgres::PgQueryResult, PgExecutor};
use uuid::Uuid;

/// A secret entry stored in the database.
#[derive(Debug)]
pub struct Secret {
    pub uuid: Uuid,
    pub file_name: Option<String>,
    pub contents: Option<Vec<u8>>,
}

impl Secret {
    /// Tries to find a Secret based on its UUID. Returns an Error if nothing
    /// could be found.
    pub async fn find<'e>(db: impl PgExecutor<'e>, uuid: Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "select uuid, file_name, contents from secrets where uuid = $1",
            uuid
        )
        .fetch_one(db)
        .await
    }

    /// Updates the contents of the Secret, both in the struct, but also in the
    /// database.
    pub async fn update_contents<'e>(
        &mut self,
        db: impl PgExecutor<'e>,
        contents: Vec<u8>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        self.contents = Some(contents.clone());
        sqlx::query_as!(
            Self,
            "update secrets set contents = $1 where uuid = $2",
            contents,
            self.uuid
        )
        .execute(db)
        .await
    }
}

impl IntoResponse for Secret {
    /// Simpl [IntoResponse] implementation for the Secret. Will return an empty
    /// response with a 204 status code if there is no content. If there is
    /// content, it will respond with it. The `content-disposition` header will
    /// contain the target filename if the field is set in the database.
    fn into_response(self) -> axum::response::Response {
        let dispo_header = match &self.file_name {
            None => "attachment".to_string(),
            Some(file_name) => format!(r#"attachment; filename="{}""#, file_name),
        };

        match self.contents {
            None => Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty()),
            Some(contents) => Response::builder()
                .status(StatusCode::OK)
                .header("content-disposition", dispo_header)
                .body(Body::from(contents)),
        }
        .expect("Response Builder with known setup should not fail")
    }
}
