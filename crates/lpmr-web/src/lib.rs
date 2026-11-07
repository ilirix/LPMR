mod jobs;
mod routes;
mod state;

use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn serve(bind: SocketAddr, assets_dir: std::path::PathBuf) -> Result<()> {
    let app = Router::new()
        .merge(routes::api_router())
        .merge(routes::static_router(assets_dir.clone()))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state::AppState::new(assets_dir.clone()));

    info!("LPMR web panel listening on http://{bind}");
    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
