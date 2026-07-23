use crate::db::Database;
use crate::parser::extract_domain;
use rand::seq::SliceRandom;
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
    pub modifiers: Vec<String>,
    pub max_results_per_query: usize,
}

impl Default for DiscoveryStrategy {
    fn default() -> Self {
        Self {
            countries: vec![
                "US".to_string(), "UK".to_string(), "Canada".to_string(), "Australia".to_string(),
                "Germany".to_string(), "Netherlands".to_string(), "France".to_string(), "Switzerland".to_string(),
                "Singapore".to_string(), "UAE".to_string(), "Ireland".to_string(), "Sweden".to_string(),
                "Denmark".to_string(), "Norway".to_string(), "Finland".to_string(), "Austria".to_string(),
                "Belgium".to_string(), "Spain".to_string(), "Italy".to_string(), "Poland".to_string(),
                "Japan".to_string(), "Israel".to_string(), "Saudi Arabia".to_string(), "Qatar".to_string(),
                "India".to_string(), "Mexico".to_string(), "Brazil".to_string(), "South Africa".to_string(),
                "Estonia".to_string(), "New Zealand".to_string(),
            ],
            industries: vec![
                "software development".to_string(),
                "cloud consulting".to_string(),
                "managed IT services".to_string(),
                "AI consulting".to_string(),
                "DevOps engineering".to_string(),
                "cybersecurity firm".to_string(),
                "custom software agency".to_string(),
                "data analytics consultancy".to_string(),
                "mobile app development".to_string(),
                "SaaS development".to_string(),
                "enterprise software".to_string(),
                "web development company".to_string(),
                "fintech software".to_string(),
                "healthcare IT".to_string(),
                "e-commerce development".to_string(),
                "IT infrastructure".to_string(),
                "blockchain engineering".to_string(),
                "QA automation services".to_string(),
                "digital transformation consultancy".to_string(),
                "IT outsourcing provider".to_string(),
                "offshore development team".to_string(),
                "staff augmentation IT".to_string(),
                "cloud migration services".to_string(),
                "UI UX design agency".to_string(),
                "IoT software solutions".to_string(),
                // Corporate Tool Searching & Procurement Keywords
                "Enterprise software procurement".to_string(),
                "SaaS vendor evaluation".to_string(),
                "IT tool consolidation platforms".to_string(),
                "Technology stack optimization".to_string(),
                // Budgeting & Finance
                "Software ROI calculator".to_string(),
                "IT budget allocation software".to_string(),
                "Software license management".to_string(),
                // Scale Keywords
                "Enterprise companies IT".to_string(),
                "Fortune 500 tech stack".to_string(),
                "Mid-market IT solutions".to_string(),
                "High-growth startups software".to_string(),
                // Targeted Titles & Tech Stack Keywords
                "CTO Salesforce".to_string(),
                "CIO AWS GCP Azure".to_string(),
                "Director of Engineering AWS".to_string(),
                "VP of Engineering Kubernetes".to_string(),
                "Head of DevOps Terraform".to_string(),
                "Director of Procurement IT".to_string(),
                "Strategic Sourcing Manager SaaS".to_string(),
            ],
            cities: vec![
                // US Major Tech Hubs
                "New York".to_string(), "San Francisco".to_string(), "Chicago".to_string(), "Houston".to_string(),
                "Austin".to_string(), "Boston".to_string(), "Seattle".to_string(), "Denver".to_string(),
                "Atlanta".to_string(), "Dallas".to_string(), "Miami".to_string(), "Los Angeles".to_string(),
                "San Jose".to_string(), "San Diego".to_string(), "Philadelphia".to_string(), "Phoenix".to_string(),
                "Raleigh".to_string(), "Charlotte".to_string(), "Indianapolis".to_string(), "Columbus".to_string(),
                "Minneapolis".to_string(), "Portland".to_string(), "Salt Lake City".to_string(), "Nashville".to_string(),
                "Tampa".to_string(), "Detroit".to_string(), "Baltimore".to_string(), "Pittsburgh".to_string(),
                "Orlando".to_string(), "San Antonio".to_string(), "Sacramento".to_string(), "Kansas City".to_string(),
                // UK Major Tech Hubs
                "London".to_string(), "Manchester".to_string(), "Birmingham".to_string(), "Edinburgh".to_string(),
                "Bristol".to_string(), "Glasgow".to_string(), "Leeds".to_string(), "Liverpool".to_string(),
                "Newcastle".to_string(), "Sheffield".to_string(), "Belfast".to_string(), "Cambridge".to_string(),
                "Oxford".to_string(), "Nottingham".to_string(), "Cardiff".to_string(), "Southampton".to_string(),
                // Global Tech Hubs
                "Toronto".to_string(), "Vancouver".to_string(), "Montreal".to_string(), "Sydney".to_string(),
                "Melbourne".to_string(), "Brisbane".to_string(), "Berlin".to_string(), "Munich".to_string(),
                "Hamburg".to_string(), "Frankfurt".to_string(), "Amsterdam".to_string(), "Rotterdam".to_string(),
                "Dublin".to_string(), "Stockholm".to_string(), "Gothenburg".to_string(), "Zurich".to_string(),
                "Geneva".to_string(), "Singapore".to_string(), "Dubai".to_string(), "Abu Dhabi".to_string(),
                "Paris".to_string(), "Lyon".to_string(), "Madrid".to_string(), "Barcelona".to_string(),
                "Milan".to_string(), "Oslo".to_string(), "Copenhagen".to_string(), "Helsinki".to_string(),
                "Vienna".to_string(), "Brussels".to_string(), "Warsaw".to_string(), "Krakow".to_string(),
                "Tokyo".to_string(), "Tel Aviv".to_string(), "Riyadh".to_string(), "Bengaluru".to_string(),
                "Mumbai".to_string(), "Delhi NCR".to_string(), "Pune".to_string(), "Hyderabad".to_string(),
                "Mexico City".to_string(), "Sao Paulo".to_string(), "Cape Town".to_string(), "Tallinn".to_string(),
                "Auckland".to_string(),
            ],
            modifiers: vec![
                "best".to_string(), "top".to_string(), "leading".to_string(), "company".to_string(),
                "agency".to_string(), "firm".to_string(), "services".to_string(), "solutions".to_string(),
                "consulting".to_string(), "contractors".to_string(),
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
    "duckduckgo.com", "clutch.co", "upwork.com", "fiverr.com", "goodfirms.co",
    "yelp.com", "trustpilot.com", "g2.com", "capterra.com", "zoominfo.com",
];

pub struct AutoSeedDiscovery {
    db: Database,
}

impl AutoSeedDiscovery {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Dynamically synthesizes search queries across global markets and fetches live company domains
    pub async fn discover_live_seeds(&self, strategy: Option<DiscoveryStrategy>) -> Vec<String> {
        let strat = strategy.unwrap_or_default();
        let mut discovered_seeds = HashSet::new();
        let client = Client::builder()
            .timeout(Duration::from_secs(12))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();

        let mut query_pool = Vec::new();

        {
            let mut rng = rand::thread_rng();
            // Sample query matrix combinations pseudo-randomly
            for _ in 0..30 {
                if let (Some(country), Some(industry), Some(city), Some(modifier)) = (
                    strat.countries.choose(&mut rng),
                    strat.industries.choose(&mut rng),
                    strat.cities.choose(&mut rng),
                    strat.modifiers.choose(&mut rng),
                ) {
                    let query_str = format!("{} {} in {} {}", modifier, industry, city, country);
                    query_pool.push(query_str);
                }
            }
        }

        let mut executed_count = 0;

        for query_str in query_pool {
            if executed_count >= 5 {
                break;
            }

            if self.db.is_query_executed(&query_str).unwrap_or(false) {
                continue;
            }

            executed_count += 1;
            let _ = self.db.mark_query_executed(&query_str);
            let _ = self.db.log_event("INFO", "DISCOVERY", &format!("Executing live web query: '{}'", query_str));

            // Engine 1: DuckDuckGo HTML
            let ddg_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(&query_str));
            if let Ok(resp) = client.get(&ddg_url).header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8").send().await {
                if resp.status().is_success() {
                    if let Ok(html_text) = resp.text().await {
                        let doc = Html::parse_document(&html_text);
                        let selectors = vec!["a.result__a", "a.result__url", "a.result__title", "a.result__snippet", "a[href*='uddg=']"];
                        for sel_str in selectors {
                            if let Ok(selector) = Selector::parse(sel_str) {
                                for el in doc.select(&selector) {
                                    let href = el.value().attr("href").unwrap_or("");
                                    let raw_url = if href.contains("uddg=") {
                                        let clean_part = href.split("uddg=").nth(1).unwrap_or("").split('&').next().unwrap_or("");
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

            // Engine 2: DuckDuckGo Lite Fallback
            if discovered_seeds.len() < 3 {
                let ddg_lite_url = format!("https://lite.duckduckgo.com/lite/?q={}", urlencoding::encode(&query_str));
                if let Ok(resp) = client.get(&ddg_lite_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(html_text) = resp.text().await {
                            let doc = Html::parse_document(&html_text);
                            if let Ok(selector) = Selector::parse("a.result-link") {
                                for el in doc.select(&selector) {
                                    let href = el.value().attr("href").unwrap_or("");
                                    if let Some(domain) = extract_domain(href) {
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

        // If queries were all previously marked executed or returned few targets, reset query table
        if executed_count == 0 {
            let _ = self.db.clear_executed_queries();
            let _ = self.db.log_event("INFO", "DISCOVERY", "Exhausted query matrix batch. Reset executed queries for fresh cycle.");
        }

        // Fallback 1: Pull un-crawled pending URLs from recursive Graph Crawl Queue
        if discovered_seeds.len() < 5 {
            if let Ok(queue_urls) = self.db.pop_pending_queue_domains(25) {
                for u in queue_urls {
                    if let Some(d) = extract_domain(&u) {
                        if self.is_valid_company_domain(&d) && !self.db.is_domain_crawled(&d).unwrap_or(false) {
                            discovered_seeds.insert(u);
                        }
                    }
                }
            }
        }

        // Fallback 2: Extended static B2B company seed targets
        let static_seeds = vec![
            "https://thoughtworks.com", "https://endava.com", "https://epam.com",
            "https://globant.com", "https://kinandcarta.com", "https://nearform.com",
            "https://boldare.com", "https://cleveroad.com", "https://eleks.com",
            "https://n-iX.com", "https://intellectsoft.net", "https://bairesdev.com",
            "https://sumato-soft.com", "https://netguru.com", "https://infinum.com",
            "https://valuecoders.com", "https://mindfire-solutions.com", "https://radixweb.com",
            "https://hiddenbrains.com", "https://bacancytechnology.com", "https://tatvasoft.com",
            "https://spec-india.com", "https://specbee.com", "https://clariontech.com",
            "https://fingent.com", "https://promptcloud.com", "https://simform.com",
            "https://ineuron.ai", "https://turing.com", "https://andela.com",
            "https://toptal.com", "https://arc.dev", "https://crossover.com",
        ];

        for s in &static_seeds {
            if let Some(domain) = extract_domain(s) {
                if self.is_valid_company_domain(&domain) && !self.db.is_domain_crawled(&domain).unwrap_or(false) {
                    discovered_seeds.insert(s.to_string());
                }
            }
        }

        // Fallback 3: Infinite Dynamic B2B Seed Generator if count is still low
        if discovered_seeds.len() < 10 {
            let prefixes = vec![
                "apex", "vanguard", "nexus", "horizon", "beacon", "crest", "summit", "matrix",
                "omni", "pinnacle", "vertex", "zenith", "quantum", "strata", "synergy", "optima",
                "velocity", "infinitum", "vector", "cyber", "cloud", "data", "tech", "logic",
                "soft", "sys", "digital", "labs", "studio", "networks", "interactive", "solutions",
                "blue", "red", "green", "silver", "gold", "iron", "steel", "titan", "phoenix",
            ];
            let suffixes = vec![
                "tech", "software", "solutions", "labs", "digital", "systems", "consulting",
                "group", "partners", "networks", "logic", "cloud", "ai", "dev", "ops",
            ];
            let tlds = vec!["com", "co.uk", "io", "tech", "dev", "ai"];

            for p in &prefixes {
                if discovered_seeds.len() >= 25 { break; }
                for s in &suffixes {
                    if discovered_seeds.len() >= 25 { break; }
                    for tld in &tlds {
                        let dom = format!("{}{}.{}", p, s, tld);
                        if self.is_valid_company_domain(&dom) && !self.db.is_domain_crawled(&dom).unwrap_or(false) {
                            discovered_seeds.insert(format!("https://{}", dom));
                            break;
                        }
                    }
                }
            }
        }

        if discovered_seeds.is_empty() {
            for s in &static_seeds {
                discovered_seeds.insert(s.to_string());
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

    pub fn generate_validated_seeds(&self, _strategy: Option<DiscoveryStrategy>) -> Vec<String> {
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
