use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub domain: String,
    pub website: String,
    pub country: String,
    pub city: Option<String>,
    pub industry: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub contact_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub hiring: bool,
    pub engineering_jobs: usize,
    pub remote_jobs: usize,
    pub outsourcing_keywords: usize,
    pub lead_score: i32,
    pub priority_tier: String, // "HIGH", "MEDIUM", "LOW"
    pub tech_stack: Vec<String>,
    pub qualification_stage: String, // "DISCOVERED", "ENRICHED", "CONTACTED", "QUALIFIED", "PROPOSAL", "WON"
    pub contact_person: Option<String>,
    pub contact_position: Option<String>,
    pub last_crawled: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLeadStageRequest {
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: i64,
    pub company_id: i64,
    pub company_name: String,
    pub company_domain: String,
    pub name: String,
    pub title: String,
    pub normalized_role: String, // "Technology Executive", "Engineering Leadership", "Executive Management"
    pub decision_maker_score: i32, // 0 - 100
    pub public_email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub confidence_score: i32,
}

fn default_page() -> usize { 1 }
fn default_limit() -> usize { 25 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonFilter {
    pub min_score: Option<i32>,
    pub role: Option<String>,
    pub domain: Option<String>,
    pub has_email: Option<bool>,
    pub search_query: Option<String>,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investor {
    pub id: i64,
    pub name: String,
    pub investor_type: String,
    pub website: String,
    pub country: String,
    pub city: Option<String>,
    pub focus: Vec<String>,
    pub stages: Vec<String>,
    pub check_size: Option<String>,
    pub public_email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub portfolio_highlights: Vec<String>,
    pub recent_investments: usize,
    pub score: i32,
    pub priority_tier: String,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadFilter {
    pub min_score: Option<i32>,
    pub country: Option<String>,
    pub hiring_only: Option<bool>,
    pub has_email: Option<bool>,
    pub search_query: Option<String>,
    pub priority: Option<String>,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestorFilter {
    pub min_score: Option<i32>,
    pub country: Option<String>,
    pub investor_type: Option<String>,
    pub focus: Option<String>,
    pub stage: Option<String>,
    pub has_email: Option<bool>,
    pub search_query: Option<String>,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestorMatchRequest {
    pub company_name: String,
    pub sectors: Vec<String>,
    pub target_market: Vec<String>,
    pub funding_stage: String,
    pub funding_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestorMatchResult {
    pub investor: Investor,
    pub match_score: i32,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadScoreBreakdown {
    pub base_score: i32,
    pub hiring_bonus: i32,
    pub engineering_jobs_score: i32,
    pub remote_jobs_score: i32,
    pub outsourcing_intent_score: i32,
    pub contact_info_bonus: i32,
    pub total_score: i32,
    pub signals_found: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerStats {
    pub total_companies: usize,
    pub high_intent_leads: usize,
    pub medium_intent_leads: usize,
    pub hiring_companies: usize,
    pub leads_with_email: usize,
    pub leads_with_phone: usize,
    pub total_decision_makers: usize,
    pub total_investors: usize,
    pub tier1_investors: usize,
    pub total_crawled_pages: usize,
    pub active_proxies: usize,
    pub crawler_status: String,
    pub current_domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub id: i64,
    pub url: String,
    pub protocol: String,
    pub active: bool,
    pub success_count: usize,
    pub fail_count: usize,
    pub latency_ms: u64,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProxyRequest {
    pub proxies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrawlSeedRequest {
    #[serde(default)]
    pub seed_urls: Vec<String>,
    pub keywords: Option<Vec<String>>,
    pub countries: Option<Vec<String>>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerSettings {
    pub mode: String,
    pub max_pages_per_domain: usize,
    pub concurrency_limit: usize,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub user_agent_rotation: bool,
    pub proxy_rotation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlLogEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub domain: String,
    pub message: String,
}
