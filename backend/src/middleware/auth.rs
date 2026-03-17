// backend/src/middleware/auth.rs
use axum::{extract::{Request, State}, middleware::Next, response::Response};
use crate::{auth::verify_token, error::AppError, state::AppState};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> std::result::Result<Response, AppError> {
    let header = req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = header.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;
    let claims = verify_token(&state.config.jwt_secret, token)
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
