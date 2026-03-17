// backend/src/error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")] Db(#[from] sqlx::Error),
    #[error("Not found")]           NotFound,
    #[error("Unauthorized")]        Unauthorized,
    #[error("Forbidden")]           Forbidden,
    #[error("Bad request: {0}")]    BadRequest(String),
    #[error("Internal: {0}")]       Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Self::NotFound     => (StatusCode::NOT_FOUND,            self.to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED,         self.to_string()),
            Self::Forbidden    => (StatusCode::FORBIDDEN,            self.to_string()),
            Self::BadRequest(m)=> (StatusCode::BAD_REQUEST,          m.clone()),
            Self::Db(_)        => (StatusCode::INTERNAL_SERVER_ERROR,"Database error".into()),
            Self::Internal(_)  => (StatusCode::INTERNAL_SERVER_ERROR,"Internal error".into()),
        };
        (status, Json(serde_json::json!({ "message": msg }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
