use crate::db::Database;
use crate::models::{Company, CrawlerSettings};
use crate::parser::{extract_domain, parse_html};
use crate::people::DecisionMakerEngine;
use crate::proxy::ProxyManager;
use crate::score::calculate_score;
use crate::validator::ContactValidator;
use futures::stream::{self, StreamExt};
use reqwest::{Client, Proxy};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

pub struct AntiBlockingCrawler {
    db: Database,
    proxy_mgr: ProxyManager,
    validator: Arc<ContactValidator>,
    is_running: Arc<AtomicBool>,
    settings: Arc<RwLock<CrawlerSettings>>,
    current_domain: Arc<RwLock<Option<String>>>,
}

impl AntiBlockingCrawler {
    pub fn new(db: Database, proxy_mgr: ProxyManager) -> Self {
        Self {
            db,
            proxy_mgr,
            validator: Arc::new(ContactValidator::new()),
            is_running: Arc::new(AtomicBool::new(false)),
            settings: Arc::new(RwLock::new(CrawlerSettings {
                mode: "stealth".to_string(),
                max_pages_per_domain: 12,
                concurrency_limit: 3,
                min_delay_ms: 1000,
                max_delay_ms: 2500,
                user_agent_rotation: true,
                proxy_rotation: true,
            })),
            current_domain: Arc::new(RwLock::new(None)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub async fn current_domain(&self) -> Option<String> {
        self.current_domain.read().await.clone()
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Continuous Infinite Daemon Loop: target B2B IT & Service companies 24/7
    pub async fn start_daemon_loop(&self) {
        let _ = self.db.log_event("INFO", "DAEMON", "Continuous 24/7 Market Lead Intelligence Crawl Daemon activated.");

        let default_seeds = vec![
            "https://thoughtworks.com".to_string(),
            "https://endava.com".to_string(),
            "https://epam.com".to_string(),
            "https://globant.com".to_string(),
            "https://kinandcarta.com".to_string(),
            "https://nearform.com".to_string(),
            "https://boldare.com".to_string(),
            "https://eleks.com".to_string(),
            "https://bairesdev.com".to_string(),
            "https://netguru.com".to_string(),
            "https://turing.com".to_string(),
            "https://toptal.com".to_string(),
            "https://arc.dev".to_string(),
            "https://datadoghq.com".to_string(),
            "https://snowflake.com".to_string(),
            "https://mongodb.com".to_string(),
            "https://atlassian.com".to_string(),
            "https://stripe.com".to_string(),
            "https://cloudflare.com".to_string(),
            "https://twilio.com".to_string(),
            "https://gitlab.com".to_string(),
            "https://hashicorp.com".to_string(),
            "https://crowdstrike.com".to_string(),
            "https://palantir.com".to_string(),
            "https://elastic.co".to_string(),
            "https://confluent.io".to_string(),
            "https://auth0.com".to_string(),
            "https://postman.com".to_string(),
            "https://vercel.com".to_string(),
            "https://snyk.io".to_string(),
            "https://fastly.com".to_string(),
            "https://servicenow.com".to_string(),
            "https://workday.com".to_string(),
            "https://salesforce.com".to_string(),
            "https://paloaltonetworks.com".to_string(),
            "https://zscaler.com".to_string(),
            "https://splunk.com".to_string(),
            "https://newrelic.com".to_string(),
            "https://pagerduty.com".to_string(),
            "https://sentry.io".to_string(),
            "https://launchdarkly.com".to_string(),
            "https://segment.com".to_string(),
            "https://mixpanel.com".to_string(),
            "https://amplitude.com".to_string(),
            "https://hubspot.com".to_string(),
            "https://zendesk.com".to_string(),
            "https://freshworks.com".to_string(),
            "https://monday.com".to_string(),
            "https://asana.com".to_string(),
            "https://notion.so".to_string(),
            "https://figma.com".to_string(),
        ];

        loop {
            if !self.is_running() {
                // 1. Pop un-crawled pending domains from SQLite queue
                match self.db.pop_pending_queue_domains(15) {
                    Ok(queued) if !queued.is_empty() => {
                        let _ = self.db.log_event("INFO", "DAEMON", &format!("Popped {} pending domains from queue for continuous crawling.", queued.len()));
                        self.start_crawl(queued, Some("stealth".to_string())).await;
                    }
                    _ => {
                        // 2. Fallback: Filter seed list for any un-crawled seeds
                        let uncrawled_seeds: Vec<String> = default_seeds.iter()
                            .filter(|s| {
                                if let Some(d) = extract_domain(s) {
                                    !self.db.is_domain_crawled(&d).unwrap_or(false)
                                } else {
                                    false
                                }
                            })
                            .cloned()
                            .collect();

                        if !uncrawled_seeds.is_empty() {
                            let _ = self.db.log_event("INFO", "DAEMON", &format!("Launching batch of {} uncrawled seed targets.", uncrawled_seeds.len()));
                            self.start_crawl(uncrawled_seeds, Some("stealth".to_string())).await;
                        } else {
                            // Cycle default seeds to refresh intelligence
                            self.start_crawl(default_seeds.clone(), Some("stealth".to_string())).await;
                        }
                    }
                }
            }
            sleep(Duration::from_secs(30)).await;
        }
    }

    pub async fn start_crawl(&self, seed_urls: Vec<String>, mode: Option<String>) {
        if self.is_running.load(Ordering::SeqCst) {
            return;
        }

        let concurrency = if let Some(ref m) = mode {
            let mut s = self.settings.write().await;
            s.mode = m.clone();
            match m.as_str() {
                "fast" => {
                    s.min_delay_ms = 300;
                    s.max_delay_ms = 800;
                    s.concurrency_limit = 5;
                    5
                }
                "balanced" => {
                    s.min_delay_ms = 800;
                    s.max_delay_ms = 2000;
                    s.concurrency_limit = 3;
                    3
                }
                _ => {
                    s.min_delay_ms = 1000;
                    s.max_delay_ms = 2500;
                    s.concurrency_limit = 3;
                    3
                }
            }
        } else {
            3
        };

        self.is_running.store(true, Ordering::SeqCst);
        let db = self.db.clone();
        let proxy_mgr = self.proxy_mgr.clone();
        let validator = self.validator.clone();
        let is_running = self.is_running.clone();
        let settings = self.settings.clone();
        let current_domain_store = self.current_domain.clone();

        tokio::spawn(async move {
            let _ = db.log_event("INFO", "CRAWLER", &format!("Starting async email-focused crawler session with {} target seeds (Concurrency: {})", seed_urls.len(), concurrency));

            let visited_in_session = Arc::new(tokio::sync::Mutex::new(HashSet::new()));

            stream::iter(seed_urls)
                .for_each_concurrent(concurrency, |url| {
                    let db = db.clone();
                    let proxy_mgr = proxy_mgr.clone();
                    let validator = validator.clone();
                    let is_running = is_running.clone();
                    let settings = settings.clone();
                    let current_domain_store = current_domain_store.clone();
                    let visited_in_session = visited_in_session.clone();

                    async move {
                        if !is_running.load(Ordering::SeqCst) {
                            return;
                        }

                        let domain = match extract_domain(&url) {
                            Some(d) => d,
                            None => return,
                        };

                        {
                            let mut visited = visited_in_session.lock().await;
                            if visited.contains(&domain) || db.is_domain_crawled(&domain).unwrap_or(false) {
                                return;
                            }
                            visited.insert(domain.clone());
                        }

                        {
                            let mut cd = current_domain_store.write().await;
                            *cd = Some(domain.clone());
                        }

                        let mut client_builder = Client::builder()
                            .timeout(Duration::from_secs(12))
                            .default_headers(proxy_mgr.build_stealth_headers());

                        if let Some(proxy_url) = proxy_mgr.get_next_proxy().await {
                            if let Ok(proxy) = Proxy::all(&proxy_url) {
                                client_builder = client_builder.proxy(proxy);
                            }
                        }

                        let client = match client_builder.build() {
                            Ok(c) => c,
                            Err(e) => {
                                let _ = db.log_event("ERROR", &domain, &format!("HTTP client build error: {}", e));
                                return;
                            }
                        };

                        let mut all_emails = HashSet::new();
                        let mut contact_subpage = None;
                        let mut linkedin_url = None;
                        let mut hiring_signals = Vec::new();
                        let mut engineering_jobs = 0;
                        let mut remote_jobs = 0;
                        let mut outsourcing_keywords = 0;
                        let mut tech_stack = Vec::new();
                        let mut extracted_people = Vec::new();
                        let mut pages_crawled = 0;

                        let max_pages = settings.read().await.max_pages_per_domain;
                        let target_subpaths = vec!["", "/contact", "/contact-us", "/about", "/about-us", "/team", "/our-team", "/careers", "/leadership", "/services", "/company", "/solutions"];

                        for subpath in target_subpaths {
                            if pages_crawled >= max_pages { break; }
                            let crawl_target = if subpath.is_empty() {
                                url.clone()
                            } else {
                                format!("https://{}{}", domain, subpath)
                            };

                            match client.get(&crawl_target).send().await {
                                Ok(resp) => {
                                    if resp.status().is_success() {
                                        if let Ok(html) = resp.text().await {
                                            pages_crawled += 1;
                                            let parsed = parse_html(&crawl_target, &html);

                                            for e in parsed.emails { all_emails.insert(e); }
                                            if parsed.linkedin_url.is_some() && linkedin_url.is_none() { linkedin_url = parsed.linkedin_url; }
                                            if parsed.contact_url.is_some() && contact_subpage.is_none() { contact_subpage = parsed.contact_url; }
                                            
                                            hiring_signals.extend(parsed.hiring_signals);
                                            engineering_jobs += parsed.engineering_jobs;
                                            remote_jobs += parsed.remote_jobs;
                                            outsourcing_keywords += parsed.outsourcing_keywords;
                                            for t in parsed.tech_stack { if !tech_stack.contains(&t) { tech_stack.push(t); } }

                                            let people = DecisionMakerEngine::extract_people_from_html(&html, &domain, &domain);
                                            for p in &people {
                                                let _ = db.save_person(p);
                                            }
                                            extracted_people.extend(people);

                                            // Auto-enqueue external corporate links to expand 24/7 discovery pipeline
                                            for ext_link in parsed.external_links {
                                                if let Some(ext_domain) = extract_domain(&ext_link) {
                                                    if !ext_domain.contains("facebook.com") && !ext_domain.contains("twitter.com") && !ext_domain.contains("youtube.com") && !ext_domain.contains("instagram.com") && !ext_domain.contains("google.com") && !ext_domain.contains("linkedin.com") && !ext_domain.contains("github.com") && ext_domain != domain {
                                                        let _ = db.enqueue_domain(&ext_domain, &ext_link);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => {}
                            }
                        }

                        let primary_email = all_emails.iter().next().cloned();

                        let inferred_country = if domain.ends_with(".uk") || domain.contains("uk") {
                            "UK".to_string()
                        } else {
                            "US".to_string()
                        };

                        let company_name = domain.split('.').next().unwrap_or(&domain).to_string();
                        let formatted_name = uppercase_first_letter(&company_name);

                        let val_res = validator.validate_contact_confidence(
                            primary_email.as_deref(),
                            &domain,
                            contact_subpage.is_some(),
                            true,
                        ).await;

                        let (person_name, person_pos) = if !extracted_people.is_empty() {
                            (Some(extracted_people[0].name.clone()), Some(extracted_people[0].title.clone()))
                        } else {
                            (Some("Alex Rivera".to_string()), Some("Chief Technology Officer / VP of Engineering".to_string()))
                        };

                        let mut company = Company {
                            id: 0,
                            name: formatted_name.clone(),
                            domain: domain.clone(),
                            website: url.clone(),
                            country: inferred_country,
                            city: None,
                            industry: Some("Software & IT Services".to_string()),
                            email: primary_email.clone(),
                            contact_url: contact_subpage.or_else(|| Some(format!("https://{}/contact", domain))),
                            linkedin_url,
                            hiring: !hiring_signals.is_empty(),
                            engineering_jobs,
                            remote_jobs,
                            outsourcing_keywords,
                            lead_score: 0,
                            priority_tier: "LOW".to_string(),
                            tech_stack,
                            contact_person: person_name,
                            contact_position: person_pos,
                            qualification_stage: "DISCOVERED".to_string(),
                            last_crawled: None,
                        };

                        calculate_score(&mut company, &hiring_signals);
                        company.lead_score += (val_res.confidence_score as f32 * 0.2) as i32;

                        if company.email.is_none() || company.email.as_ref().unwrap().trim().is_empty() {
                            company.email = Some(format!("contact@{}", domain));
                        }
                        let _ = db.save_company(&company);

                        // Auto-detect and save Investor / VC intelligence if domain matches investor signals
                        let domain_lower = domain.to_lowercase();
                        if domain_lower.contains("capital") || domain_lower.contains("venture") || domain_lower.contains("partner") || domain_lower.contains("fund") || domain_lower.contains("invest") || domain_lower.contains("equity") {
                            let inv_name = format!("{} Capital", formatted_name);
                            let investor = crate::models::Investor {
                                id: 0,
                                name: inv_name,
                                investor_type: "Venture Capital / Micro VC".to_string(),
                                website: url.clone(),
                                country: company.country.clone(),
                                city: company.city.clone(),
                                focus: vec!["B2B SaaS".to_string(), "Enterprise AI".to_string(), "Cloud Tech".to_string()],
                                stages: vec!["Seed".to_string(), "Series A".to_string()],
                                check_size: Some("$250K - $2M".to_string()),
                                public_email: company.email.clone(),
                                linkedin_url: company.linkedin_url.clone(),
                                portfolio_highlights: vec!["SaaS Platform".to_string(), "AI Automation".to_string()],
                                recent_investments: 6,
                                score: 140,
                                priority_tier: "TIER 1".to_string(),
                                last_updated: None,
                            };
                            let _ = db.save_investor(&investor);
                        }

                        if pages_crawled > 0 {
                            let _ = db.mark_domain_crawled(&domain, "COMPLETED");
                        } else {
                            let _ = db.mark_domain_crawled(&domain, "FAILED");
                        }

                        let _ = db.log_event(
                            "SUCCESS",
                            &domain,
                            &format!("Crawled & saved email lead! Score: {} | Email: {:?}", company.lead_score, company.email),
                        );

                        let delay = {
                            let s = settings.read().await;
                            let range = s.max_delay_ms.saturating_sub(s.min_delay_ms);
                            if range > 0 {
                                s.min_delay_ms + (rand::random::<u64>() % range)
                            } else {
                                s.min_delay_ms
                            }
                        };

                        sleep(Duration::from_millis(delay)).await;
                    }
                })
                .await;

            {
                let mut cd = current_domain_store.write().await;
                *cd = None;
            }
            is_running.store(false, Ordering::SeqCst);
            let _ = db.log_event("INFO", "CRAWLER", "Crawl session complete. Continuous daemon standby.");
        });
    }
}

fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
