mod api;
mod bounce_monitor;
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
mod search_utility;

use api::{create_router, AppState};
use bounce_monitor::BounceMonitorEngine;
use crawler::AntiBlockingCrawler;
use db::Database;
use proxy::ProxyManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
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
    proxy_mgr.load_proxies_from_db(&db).await;

    let crawler = Arc::new(AntiBlockingCrawler::new(db.clone(), proxy_mgr.clone()));

    let crawler_daemon = crawler.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        crawler_daemon.start_daemon_loop().await;
    });

    proxy::ProxyManager::start_proxy_health_checker(proxy_mgr.clone(), db.clone());

    let bounce_db = db.clone();
    tokio::spawn(async move {
        BounceMonitorEngine::start_daemon_loop(bounce_db).await;
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

    let compression_layer = tower_http::compression::CompressionLayer::new();
    let cache_layer = tower_http::set_header::SetResponseHeaderLayer::overriding(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );

    let app = create_router(app_state)
        .fallback(static_file_handler)
        .layer(cors)
        .layer(compression_layer)
        .layer(cache_layer);

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

async fn static_file_handler(axum::extract::OriginalUri(uri): axum::extract::OriginalUri) -> Response {
    use axum::response::IntoResponse;
    let raw_path = uri.path().trim_start_matches('/');
    let clean_path = if raw_path.is_empty() || raw_path == "index.html" {
        "static/index.html".to_string()
    } else {
        format!("static/{}", raw_path)
    };

    let path = std::path::Path::new(&clean_path);
    let (mime_type, file_to_read) = if path.exists() && path.is_file() {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let mime = match ext {
            "js" => "application/javascript; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "html" => "text/html; charset=utf-8",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "woff2" => "font/woff2",
            _ => "text/plain",
        };
        (mime, clean_path)
    } else {
        ("text/html; charset=utf-8", "static/index.html".to_string())
    };

    match tokio::fs::read(&file_to_read).await {
        Ok(contents) => (
            StatusCode::OK,
            [
                (axum::http::header::CONTENT_TYPE, mime_type),
                (axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
            ],
            contents,
        ).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            "404 Not Found",
        ).into_response(),
    }
}
