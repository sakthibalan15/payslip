// backend/src/config.rs
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url:  String,
    pub jwt_secret:    String,
    pub smtp_host:     String,
    pub smtp_port:     u16,
    pub smtp_user:     String,
    pub smtp_password: String,
    pub from_email:    String,
    pub otp_ttl_secs:  i64,
    pub smtp_use_ssl:  bool,   // NEW: true = port 465, false = port 587
    /// When true, OTP is logged (warning) and no email is sent — for local dev without SMTP.
    pub smtp_skip:     bool,
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url:  std::env::var("DATABASE_URL")?,
            jwt_secret:    std::env::var("JWT_SECRET").unwrap_or_else(|_| "change_me".into()),
            smtp_host:     std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".into()),
            smtp_port:     std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".into()).parse()?,
            smtp_user:     std::env::var("SMTP_USER").unwrap_or_default(),
            smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            from_email:    std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@company.com".into()),
            otp_ttl_secs:  std::env::var("OTP_TTL_SECS").unwrap_or_else(|_| "300".into()).parse()?,
            smtp_use_ssl:  std::env::var("SMTP_USE_SSL").unwrap_or_else(|_| "true".into()) == "true",
            smtp_skip:     env_truthy("SMTP_SKIP"),
        })
    }
}

fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name).map(|v| v.to_lowercase()),
        Ok(v) if v == "1" || v == "true" || v == "yes"
    )
}
