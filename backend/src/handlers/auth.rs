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
    let user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1", body.email
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

    send_otp_email(&state, &body.email, &otp).await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(serde_json::json!({ "message": "OTP sent to your email" })))
}

/// POST /api/auth/verify-otp
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(body): Json<VerifyOtpRequest>,
) -> Result<Json<AuthResponse>> {
    let row = sqlx::query!(
        r#"SELECT u.id, u.name, u.email, u.role,
                  ot.otp, ot.expires_at, ot.used
           FROM users u
           JOIN otp_tokens ot ON ot.user_id = u.id
           WHERE u.email = $1"#,
        body.email
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
    use lettre::{
        message::header::ContentType,
        transport::smtp::authentication::Credentials,
        AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    };
    let email = Message::builder()
        .from(state.config.from_email.parse()?)
        .to(to.parse()?)
        .subject("Your Payslip App OTP")
        .header(ContentType::TEXT_HTML)
        .body(format!(
            "<p>Your OTP is: <strong>{otp}</strong></p><p>Valid for 5 minutes.</p>"
        ))?;
    let creds = Credentials::new(state.config.smtp_user.clone(), state.config.smtp_password.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&state.config.smtp_host)?
        .port(state.config.smtp_port).credentials(creds).build();
    mailer.send(email).await?;
    Ok(())
}
