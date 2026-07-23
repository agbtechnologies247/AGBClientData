mod api;
mod campaign;
mod crawler;
mod db;
mod exporter;
mod investor_matching;
mod models;
mod parser;
mod people;
mod proxy;
mod score;
#[cfg(test)]
mod tests;
mod validator;

use api::{create_router, AppState};
use crawler::AntiBlockingCrawler;
use db::Database;
use proxy::ProxyManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,marketing_data_crawler=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Marketing Data Crawler & Lead Intelligence Server...");

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "marketing_leads.db".into());
    let db = Database::new(&db_path)?;
    info!("Database initialized at {}", db_path);

    let initial_proxies = vec![];
    let proxy_mgr = ProxyManager::new(initial_proxies);

    let crawler = Arc::new(AntiBlockingCrawler::new(db.clone(), proxy_mgr.clone()));

    let crawler_daemon = crawler.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        crawler_daemon.start_daemon_loop().await;
    });

    let outreach_db = db.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        crate::campaign::CampaignEngine::start_hourly_outreach_daemon(outreach_db);
    });

    let app_state = AppState {
        db,
        proxy_mgr,
        crawler,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(app_state)
        .nest_service("/", ServeDir::new("static"))
        .layer(cors);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Marketing Data Crawler Web App listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
