// backend/src/middleware/auth.rs
use axum::{extract::{Request, State}, middleware::Next, response::Response};
use crate::{auth::verify_token, error::AppError, state::AppState};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> std::result::Result<Response, AppError> {
    // Support both:
    // 1) Authorization: Bearer <token> (normal API calls)
    // 2) ?token=<jwt> query param (iframe/new-tab PDF preview/download)
    let token_from_header = req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(str::to_owned);

    let token_from_query = req.uri()
        .query()
        .and_then(extract_token_from_query);

    let token = token_from_header
        .or(token_from_query)
        .ok_or(AppError::Unauthorized)?;

    let claims = verify_token(&state.config.jwt_secret, &token)
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn extract_token_from_query(query: &str) -> Option<String> {
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("token="))
        .map(str::to_owned)
}
