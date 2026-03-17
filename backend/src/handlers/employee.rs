// backend/src/handlers/employee.rs
use axum::{extract::{Extension, Query, State}, response::Response};
use shared::PayslipQuery;
use crate::{auth::Claims, error::Result, handlers::admin::{fetch_payslip, attachment_pdf, inline_pdf}, pdf, state::AppState};

pub async fn preview_payslip(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<PayslipQuery>,
) -> Result<Response> {
    let data = fetch_payslip(&state, claims.sub, q.year, q.month).await?;
    Ok(inline_pdf(pdf::generate(&data)))
}

pub async fn download_payslip(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<PayslipQuery>,
) -> Result<Response> {
    let data = fetch_payslip(&state, claims.sub, q.year, q.month).await?;
    Ok(attachment_pdf(pdf::generate(&data), q.year, q.month))
}
