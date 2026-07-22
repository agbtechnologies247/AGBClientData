use crate::db::Database;
use crate::parser::extract_domain;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStrategy {
    pub countries: Vec<String>,
    pub industries: Vec<String>,
    pub cities: Vec<String>,
    pub max_results_per_query: usize,
}

impl Default for DiscoveryStrategy {
    fn default() -> Self {
        Self {
            countries: vec!["US".to_string(), "UK".to_string()],
            industries: vec![
                "software development".to_string(),
                "cloud consulting".to_string(),
                "managed IT services".to_string(),
                "AI consulting".to_string(),
                "DevOps engineering".to_string(),
            ],
            cities: vec![
                "New York".to_string(),
                "Austin".to_string(),
                "Boston".to_string(),
                "Chicago".to_string(),
                "London".to_string(),
                "Manchester".to_string(),
            ],
            max_results_per_query: 10,
        }
    }
}

const BLOCKED_DOMAINS: &[&str] = &[
    "facebook.com", "instagram.com", "twitter.com", "x.com", "linkedin.com",
    "youtube.com", "wikipedia.org", "github.com", "medium.com", "blogspot.com",
    "wordpress.com", "reddit.com", "glassdoor.com", "indeed.com", "quora.com",
    "amazon.com", "google.com", "bing.com", "yahoo.com", "apple.com", "microsoft.com",
    "duckduckgo.com", "clutch.co", "upwork.com", "fiverr.com",
];

pub struct AutoSeedDiscovery {
    db: Database,
}

impl AutoSeedDiscovery {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Dynamically synthesizes search queries and fetches live company domains via search endpoints
    pub async fn discover_live_seeds(&self, strategy: Option<DiscoveryStrategy>) -> Vec<String> {
        let strat = strategy.unwrap_or_default();
        let mut discovered_seeds = HashSet::new();
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();

        let mut query_count = 0;

        for country in &strat.countries {
            for industry in &strat.industries {
                for city in &strat.cities {
                    if query_count >= 5 { break; } // Bounded query budget per cycle

                    let query_str = format!("{} company in {} {}", industry, city, country);

                    if self.db.is_query_executed(&query_str).unwrap_or(false) {
                        continue;
                    }

                    query_count += 1;
                    let _ = self.db.mark_query_executed(&query_str);
                    let _ = self.db.log_event("INFO", "DISCOVERY", &format!("Executing live web query: '{}'", query_str));

                    // Query DuckDuckGo HTML endpoint
                    let ddg_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(&query_str));

                    if let Ok(resp) = client.get(&ddg_url).send().await {
                        if resp.status().is_success() {
                            if let Ok(html_text) = resp.text().await {
                                let doc = Html::parse_document(&html_text);
                                let link_selector = Selector::parse("a.result__url").unwrap();

                                for el in doc.select(&link_selector) {
                                    let href = el.value().attr("href").unwrap_or("");
                                    let raw_url = if href.starts_with("//duckduckgo.com/l/?uddg=") {
                                        let clean_part = href.trim_start_matches("//duckduckgo.com/l/?uddg=");
                                        urlencoding::decode(clean_part).unwrap_or_default().to_string()
                                    } else {
                                        href.to_string()
                                    };

                                    if let Some(domain) = extract_domain(&raw_url) {
                                        if self.is_valid_company_domain(&domain) && !self.db.is_domain_crawled(&domain).unwrap_or(false) {
                                            discovered_seeds.insert(format!("https://{}", domain));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback curated tech seed list if web search returned low yields
        let static_seeds = vec![
            "https://thoughtworks.com", "https://endava.com", "https://epam.com",
            "https://globant.com", "https://kinandcarta.com", "https://nearform.com",
            "https://boldare.com", "https://cleveroad.com", "https://eleks.com",
            "https://n-iX.com", "https://intellectsoft.net", "https://bairesdev.com",
            "https://sumato-soft.com", "https://netguru.com", "https://infinum.com",
        ];

        for s in static_seeds {
            if let Some(domain) = extract_domain(s) {
                if self.is_valid_company_domain(&domain) && !self.db.is_domain_crawled(&domain).unwrap_or(false) {
                    discovered_seeds.insert(s.to_string());
                }
            }
        }

        let seed_list: Vec<String> = discovered_seeds.into_iter().collect();
        let _ = self.db.log_event(
            "SUCCESS",
            "DISCOVERY",
            &format!("Live Discovery Engine synthesized {} fresh target company URLs", seed_list.len()),
        );

        seed_list
    }

    pub fn generate_validated_seeds(&self, strategy: Option<DiscoveryStrategy>) -> Vec<String> {
        let static_seeds = vec![
            "https://thoughtworks.com", "https://endava.com", "https://epam.com",
            "https://globant.com", "https://kinandcarta.com", "https://nearform.com",
            "https://boldare.com", "https://cleveroad.com", "https://eleks.com",
            "https://n-iX.com", "https://intellectsoft.net", "https://bairesdev.com",
            "https://sumato-soft.com", "https://netguru.com", "https://infinum.com",
        ];

        let mut seeds = Vec::new();
        for s in static_seeds {
            if let Some(domain) = extract_domain(s) {
                if self.is_valid_company_domain(&domain) && !self.db.is_domain_crawled(&domain).unwrap_or(false) {
                    seeds.push(s.to_string());
                }
            }
        }
        seeds
    }

    pub fn is_valid_company_domain(&self, domain: &str) -> bool {
        let clean = domain.to_lowercase();

        if BLOCKED_DOMAINS.iter().any(|b| clean == *b || clean.ends_with(&format!(".{}", b))) {
            return false;
        }

        if !clean.contains('.') {
            return false;
        }

        true
    }
}
