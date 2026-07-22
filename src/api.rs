use crate::campaign::CreateCampaignRequest;
use crate::crawler::AntiBlockingCrawler;
use crate::db::Database;
use crate::discovery::AutoSeedDiscovery;
use crate::exporter::{generate_excel_export, generate_investor_excel_export, generate_people_excel_export};
use crate::investor_matching::match_investor;
use crate::models::{AddProxyRequest, CrawlSeedRequest, InvestorFilter, InvestorMatchRequest, LeadFilter, PersonFilter};
use crate::proxy::ProxyManager;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub proxy_mgr: ProxyManager,
    pub crawler: Arc<AntiBlockingCrawler>,
    pub discovery: Arc<AutoSeedDiscovery>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/stats", get(get_stats_handler))
        .route("/api/leads", get(get_leads_handler))
        .route("/api/leads/:id", get(get_lead_by_id_handler))
        .route("/api/leads/:id/stage", post(update_lead_stage_handler))
        .route("/api/people", get(get_people_handler))
        .route("/api/people/export", get(export_people_handler))
        .route("/api/campaigns", get(get_campaigns_handler).post(create_campaign_handler))
        .route("/api/investors", get(get_investors_handler))
        .route("/api/investors/match", post(match_investors_handler))
        .route("/api/investors/export", get(export_investors_handler))
        .route("/api/crawler/start", post(start_crawler_handler))
        .route("/api/crawler/stop", post(stop_crawler_handler))
        .route("/api/crawler/auto-seeds", post(auto_discover_seeds_handler))
        .route("/api/crawler/reset-queries", post(reset_queries_handler))
        .route("/api/crawler/clear-database", post(clear_database_handler))
        .route("/api/proxies", get(get_proxies_handler).post(add_proxies_handler))
        .route("/api/logs", get(get_logs_handler))
        .route("/api/leads/export", get(export_leads_handler))
        .with_state(state)
}

async fn get_stats_handler(State(state): State<AppState>) -> Response {
    match state.db.get_stats() {
        Ok(mut stats) => {
            if state.crawler.is_running() {
                stats.crawler_status = "RUNNING".to_string();
                stats.current_domain = state.crawler.current_domain().await;
            } else {
                stats.crawler_status = "IDLE".to_string();
            }
            (StatusCode::OK, Json(stats)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_leads_handler(
    State(state): State<AppState>,
    Query(filter): Query<LeadFilter>,
) -> Response {
    match state.db.get_leads(&filter) {
        Ok((leads, total)) => (
            StatusCode::OK,
            Json(json!({
                "leads": leads,
                "total": total,
                "page": filter.page,
                "limit": filter.limit
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_lead_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Response {
    let filter = LeadFilter {
        min_score: None,
        country: None,
        hiring_only: None,
        has_email: None,
        search_query: None,
        priority: None,
        page: 1,
        limit: 1000,
    };
    if let Ok((leads, _)) = state.db.get_leads(&filter) {
        if let Some(company) = leads.into_iter().find(|c| c.id == id) {
            return (StatusCode::OK, Json(json!(company))).into_response();
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "Lead not found"}))).into_response()
}

async fn update_lead_stage_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<crate::models::UpdateLeadStageRequest>,
) -> Response {
    match state.db.update_company_qualification_stage(id, &payload.stage) {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success", "id": id, "stage": payload.stage}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_people_handler(
    State(state): State<AppState>,
    Query(filter): Query<PersonFilter>,
) -> Response {
    match state.db.get_people(&filter) {
        Ok((people, total)) => (
            StatusCode::OK,
            Json(json!({
                "people": people,
                "total": total,
                "page": filter.page,
                "limit": filter.limit
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_campaigns_handler(State(state): State<AppState>) -> Response {
    match state.db.get_campaigns() {
        Ok(campaigns) => (StatusCode::OK, Json(json!({"campaigns": campaigns}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn create_campaign_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateCampaignRequest>,
) -> Response {
    match state.db.create_campaign(&req) {
        Ok(campaign) => (StatusCode::CREATED, Json(json!(campaign))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_investors_handler(
    State(state): State<AppState>,
    Query(filter): Query<InvestorFilter>,
) -> Response {
    match state.db.get_investors(&filter) {
        Ok((investors, total)) => (
            StatusCode::OK,
            Json(json!({
                "investors": investors,
                "total": total,
                "page": filter.page,
                "limit": filter.limit
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn match_investors_handler(
    State(state): State<AppState>,
    Json(payload): Json<InvestorMatchRequest>,
) -> Response {
    let filter = InvestorFilter {
        min_score: None,
        country: None,
        investor_type: None,
        focus: None,
        stage: None,
        has_email: None,
        search_query: None,
        page: 1,
        limit: 1000,
    };

    if let Ok((investors, _)) = state.db.get_investors(&filter) {
        let mut results: Vec<_> = investors
            .iter()
            .map(|inv| match_investor(inv, &payload))
            .collect();

        results.sort_by(|a, b| b.match_score.cmp(&a.match_score));

        return (
            StatusCode::OK,
            Json(json!({
                "company_name": payload.company_name,
                "matches": results
            })),
        )
            .into_response();
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "Failed to query investors"})),
    )
        .into_response()
}

async fn start_crawler_handler(
    State(state): State<AppState>,
    payload: Option<Json<CrawlSeedRequest>>,
) -> Response {
    let req = payload.map(|p| p.0).unwrap_or_default();
    let mut seeds = req.seed_urls;
    if seeds.is_empty() {
        seeds = state.discovery.discover_live_seeds(None).await;
    }

    state.crawler.start_crawl(seeds.clone(), req.mode).await;
    (
        StatusCode::OK,
        Json(json!({
            "status": "STARTED",
            "seed_count": seeds.len(),
            "seeds": seeds
        })),
    )
        .into_response()
}

async fn stop_crawler_handler(State(state): State<AppState>) -> Response {
    state.crawler.stop();
    (StatusCode::OK, Json(json!({"status": "STOPPED"}))).into_response()
}

async fn auto_discover_seeds_handler(State(state): State<AppState>) -> Response {
    let seeds = state.discovery.discover_live_seeds(None).await;
    (
        StatusCode::OK,
        Json(json!({
            "count": seeds.len(),
            "seeds": seeds
        })),
    )
        .into_response()
}

async fn reset_queries_handler(State(state): State<AppState>) -> Response {
    match state.db.clear_executed_queries() {
        Ok(count) => (
            StatusCode::OK,
            Json(json!({"status": "SUCCESS", "cleared_queries": count})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_proxies_handler(State(state): State<AppState>) -> Response {
    match state.db.get_proxies() {
        Ok(proxies) => (StatusCode::OK, Json(json!({"proxies": proxies}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn add_proxies_handler(
    State(state): State<AppState>,
    Json(payload): Json<AddProxyRequest>,
) -> Response {
    for p in &payload.proxies {
        let protocol = if p.starts_with("socks5") { "socks5" } else { "http" };
        let _ = state.db.add_proxy(p, protocol);
    }
    state.proxy_mgr.add_proxies(payload.proxies.clone()).await;
    (
        StatusCode::OK,
        Json(json!({
            "added": payload.proxies.len(),
            "status": "SUCCESS"
        })),
    )
        .into_response()
}

async fn get_logs_handler(State(state): State<AppState>) -> Response {
    match state.db.get_recent_logs(50) {
        Ok(logs) => (StatusCode::OK, Json(json!({"logs": logs}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn export_leads_handler(
    State(state): State<AppState>,
    Query(filter): Query<LeadFilter>,
) -> Response {
    match generate_excel_export(&state.db, &filter) {
        Ok(buffer) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    .parse()
                    .unwrap(),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"Marketing_Leads_Intelligence.xlsx\""
                    .parse()
                    .unwrap(),
            );
            (StatusCode::OK, headers, buffer).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn export_people_handler(
    State(state): State<AppState>,
    Query(filter): Query<PersonFilter>,
) -> Response {
    match generate_people_excel_export(&state.db, &filter) {
        Ok(buffer) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    .parse()
                    .unwrap(),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"Decision_Makers_Intelligence.xlsx\""
                    .parse()
                    .unwrap(),
            );
            (StatusCode::OK, headers, buffer).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn export_investors_handler(
    State(state): State<AppState>,
    Query(filter): Query<InvestorFilter>,
) -> Response {
    match generate_investor_excel_export(&state.db, &filter) {
        Ok(buffer) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    .parse()
                    .unwrap(),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"B2B_SaaS_Investors_Intelligence.xlsx\""
                    .parse()
                    .unwrap(),
            );
            (StatusCode::OK, headers, buffer).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn clear_database_handler(State(state): State<AppState>) -> Response {
    state.crawler.stop();
    match state.db.clear_all_data() {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success", "message": "Database wiped successfully. Ready for fresh crawl."}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
