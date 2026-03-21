// backend/src/handlers/auth.rs
use axum::{extract::State, Json};
use chrono::Utc;
use shared::{AuthResponse, SendOtpRequest, VerifyOtpRequest};
use crate::{auth::{create_token, generate_otp}, error::{AppError, Result}, state::AppState};

/// POST /api/auth/send-otp
pub async fn send_otp(
    State(state): State<AppState>,
    Json(body): Json<SendOtpRequest>,
) -> Result<Json<serde_json::Value>> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }

    let user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1", email
    )
    .fetch_optional(&state.pool).await?
    .ok_or_else(|| AppError::BadRequest("Email not registered".into()))?;

    let otp = generate_otp();
    let expires_at = Utc::now() + chrono::Duration::seconds(state.config.otp_ttl_secs);

    sqlx::query!(
        r#"INSERT INTO otp_tokens (user_id, otp, expires_at)
           VALUES ($1, $2, $3)
           ON CONFLICT (user_id) DO UPDATE
           SET otp = EXCLUDED.otp, expires_at = EXCLUDED.expires_at, used = false"#,
        user.id, otp, expires_at
    ).execute(&state.pool).await?;

    send_otp_email(&state, &email, &otp).await
        .map_err(|e| AppError::Internal(e))?;

    let message = if state.config.smtp_skip {
        "OTP ready (dev mode: check server logs — email not sent)".to_string()
    } else {
        "OTP sent to your email".to_string()
    };
    Ok(Json(serde_json::json!({ "message": message })))
}

/// POST /api/auth/verify-otp
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(body): Json<VerifyOtpRequest>,
) -> Result<Json<AuthResponse>> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }

    let row = sqlx::query!(
        r#"SELECT u.id, u.name, u.email, u.role,
                  ot.otp, ot.expires_at, ot.used
           FROM users u
           JOIN otp_tokens ot ON ot.user_id = u.id
           WHERE u.email = $1"#,
        email
    )
    .fetch_optional(&state.pool).await?
    .ok_or_else(|| AppError::BadRequest("No OTP requested for this email".into()))?;

    if row.used { return Err(AppError::BadRequest("OTP already used".into())); }
    if Utc::now() > row.expires_at { return Err(AppError::BadRequest("OTP expired".into())); }
    if row.otp != body.otp { return Err(AppError::BadRequest("Invalid OTP".into())); }

    sqlx::query!("UPDATE otp_tokens SET used = true WHERE user_id = $1", row.id)
        .execute(&state.pool).await?;

    let role: shared::UserRole = serde_json::from_value(serde_json::Value::String(row.role.clone()))
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid role")))?;

    let token = create_token(&state.config.jwt_secret, row.id, &row.email, role.clone())
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(AuthResponse { token, role, name: row.name }))
}

async fn send_otp_email(state: &AppState, to: &str, otp: &str) -> anyhow::Result<()> {
    if state.config.smtp_skip {
        tracing::warn!(
            email = %to,
            otp = %otp,
            "SMTP_SKIP: OTP not emailed — use this OTP to sign in"
        );
        return Ok(());
    }

    let mins = (state.config.otp_ttl_secs + 59) / 60;
    let html = format!(
        "<p>Your OTP is: <strong>{otp}</strong></p><p>Valid for {mins} minute(s).</p>"
    );
    crate::mailer::send_html_email(
        &state.mailer,
        &state.config.from_email,
        to,
        "Your Payslip App OTP",
        html,
    )
    .await
}
