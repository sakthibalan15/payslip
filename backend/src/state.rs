// backend/src/state.rs
use crate::config::Config;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool:   PgPool,
    pub config: Config,
    pub mailer:   AsyncSmtpTransport<Tokio1Executor>,  // NEW
}
impl AppState {
    pub fn new(pool: PgPool, config: Config, mailer: AsyncSmtpTransport<Tokio1Executor>) -> Self { Self { pool, config, mailer } }
}
