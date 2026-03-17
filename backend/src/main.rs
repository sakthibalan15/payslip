// backend/src/main.rs

mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod pdf;
mod state;

use axum::{middleware as axum_mw, routing::{get, post}, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg  = config::Config::from_env()?;
    let pool = db::create_pool(&cfg.database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = state::AppState::new(pool, cfg);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public
    let public = Router::new()
        .route("/api/auth/send-otp",   post(handlers::auth::send_otp))
        .route("/api/auth/verify-otp", post(handlers::auth::verify_otp));

    // Protected (JWT required)
    let protected = Router::new()
        .route("/api/admin/csv/preview",        post(handlers::admin::preview_csv))
        .route("/api/admin/csv/upload",         post(handlers::admin::upload_csv))
        .route("/api/admin/payslip/preview",    get(handlers::admin::preview_payslip))
        .route("/api/admin/payslip/download",   get(handlers::admin::download_payslip))
        .route("/api/employee/payslip/preview",  get(handlers::employee::preview_payslip))
        .route("/api/employee/payslip/download", get(handlers::employee::download_payslip))
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            middleware::auth::require_auth,
        ));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("Backend listening on http://0.0.0.0:3001");
    axum::serve(listener, app).await?;
    Ok(())
}
