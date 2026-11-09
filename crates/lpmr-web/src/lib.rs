mod convert_paths;
mod execute_auth;
mod jobs;
mod routes;
mod state;

use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use axum::http::Method;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn serve(bind: SocketAddr, assets_dir: std::path::PathBuf) -> Result<()> {
    if !bind.ip().is_loopback() && std::env::var("LPMR_EXECUTE_TOKEN").is_err() {
        tracing::warn!(
            "listening on {bind}: set LPMR_EXECUTE_TOKEN for authenticated remote execution (non-dry-run is otherwise loopback/CSRF only)"
        );
    }
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-lpmr-token"),
            axum::http::HeaderName::from_static("x-lpmr-csrf"),
        ]);
    let app = Router::new()
        .merge(routes::api_router())
        .merge(routes::static_router(assets_dir.clone()))
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state::AppState::new(assets_dir.clone()));

    info!("LPMR web panel listening on http://{bind}");
    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}
