use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

pub struct ParsedContent {
    pub emails: Vec<String>,
    pub phones: Vec<String>,
    pub linkedin_url: Option<String>,
    pub contact_url: Option<String>,
    pub internal_links: Vec<String>,
    pub external_links: Vec<String>,
    pub hiring_signals: Vec<String>,
    pub engineering_jobs: usize,
    pub remote_jobs: usize,
    pub outsourcing_keywords: usize,
    pub tech_stack: Vec<String>,
}

pub fn parse_html(base_url: &str, html_body: &str) -> ParsedContent {
    let document = Html::parse_document(html_body);
    let mut emails_set = HashSet::new();

    // 1. Expanded Email Regex & Mailto: Link Extraction
    let email_regex = Regex::new(r"(?i)[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}").unwrap();
    for mat in email_regex.find_iter(html_body) {
        let email = mat.as_str().to_lowercase();
        if is_valid_business_email(&email) {
            emails_set.insert(email);
        }
    }

    // 2. Extract from explicit mailto: links in HTML
    let a_selector = Selector::parse("a[href^='mailto:']").unwrap();
    for element in document.select(&a_selector) {
        if let Some(href) = element.value().attr("href") {
            let email_raw = href.trim_start_matches("mailto:").split('?').next().unwrap_or("");
            let clean = email_raw.trim().to_lowercase();
            if is_valid_business_email(&clean) {
                emails_set.insert(clean);
            }
        }
    }

    // 3. Expanded Phone Extraction (US, UK, and International phone formats, tel: links)
    let mut phones_set = HashSet::new();
    let phone_regex = Regex::new(r"(?:\+\d{1,3}[\s.-]?)?\(?\d{2,4}\)?[\s.-]?\d{3,4}[\s.-]?\d{3,4}").unwrap();
    for mat in phone_regex.find_iter(html_body) {
        let phone = mat.as_str().trim().to_string();
        if phone.len() >= 9 && !phone.contains('@') {
            phones_set.insert(phone);
        }
    }

    let tel_selector = Selector::parse("a[href^='tel:']").unwrap();
    for element in document.select(&tel_selector) {
        if let Some(href) = element.value().attr("href") {
            let phone_raw = href.trim_start_matches("tel:").trim().to_string();
            if phone_raw.len() >= 7 {
                phones_set.insert(phone_raw);
            }
        }
    }

    // 0. Multi-Layer Advertisement & Tracker Cracking / Stripping
    let clean_html_body = html_body
        .replace(r"(?i)<script[^>]*googlesyndication[^>]*>.*?</script>", "")
        .replace(r"(?i)<iframe[^>]*ads[^>]*>.*?</iframe>", "");

    // 4. Link Extraction (Contact, Team, Careers, Verified LinkedIn, External Links)
    let link_selector = Selector::parse("a[href]").unwrap();
    let base_parsed = Url::parse(base_url).ok();
    let mut contact_url = None;
    let mut linkedin_url = None;
    let mut internal_links_set = HashSet::new();
    let mut external_links_set = HashSet::new();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Only verified LinkedIn company or personal profile URLs
            if (href.contains("linkedin.com/company/") || href.contains("linkedin.com/in/")) && !href.contains("share") && linkedin_url.is_none() {
                let clean_li = href.split('?').next().unwrap_or(href).to_string();
                linkedin_url = Some(clean_li);
            }

            if let Some(ref base) = base_parsed {
                if let Ok(abs_url) = base.join(href) {
                    if abs_url.host_str() == base.host_str() {
                        let path = abs_url.path().to_lowercase();
                        if (path.contains("contact") || path.contains("about") || path.contains("careers") || path.contains("team") || path.contains("people") || path.contains("get-in-touch")) && contact_url.is_none() {
                            contact_url = Some(abs_url.to_string());
                        }
                        internal_links_set.insert(abs_url.to_string());
                    } else if abs_url.scheme() == "http" || abs_url.scheme() == "https" {
                        external_links_set.insert(abs_url.to_string());
                    }
                }
            }
        }
    }

    // 5. Hiring & Outsourcing Intent Analysis
    let lower_body = html_body.to_lowercase();
    let mut hiring_signals = Vec::new();
    let mut engineering_jobs = 0;
    let mut remote_jobs = 0;
    let mut outsourcing_keywords = 0;

    let hiring_terms = vec![
        "we're hiring", "careers", "open positions", "join our team", "job vacancies",
        "software engineer", "full stack developer", "backend engineer", "frontend engineer",
        "devops engineer", "engineering manager", "technical lead", "remote engineer",
    ];

    for term in &hiring_terms {
        if lower_body.contains(term) {
            hiring_signals.push(term.to_string());
            if term.contains("engineer") || term.contains("developer") {
                engineering_jobs += lower_body.matches(term).count();
            }
            if term.contains("remote") {
                remote_jobs += 1;
            }
        }
    }

    let intent_phrases = vec![
        "outsourcing", "offshore development", "development partner", "dedicated team",
        "contractors", "consulting partner", "staff augmentation", "nearshore",
    ];

    for phrase in intent_phrases {
        if lower_body.contains(phrase) {
            outsourcing_keywords += 1;
            hiring_signals.push(format!("Intent Signal: {}", phrase));
        }
    }

    // 6. Tech Stack Signals
    let mut tech_stack = Vec::new();
    let tech_keywords = vec![
        ("React", vec!["react.js", "reactjs", "react"]),
        ("Node.js", vec!["node.js", "nodejs", "express.js"]),
        ("Python", vec!["python", "django", "fastapi", "flask"]),
        ("AWS", vec!["aws", "amazon web services", "ec2", "s3"]),
        ("Azure", vec!["azure", "microsoft azure"]),
        ("GCP", vec!["google cloud", "gcp"]),
        ("Kubernetes", vec!["kubernetes", "k8s"]),
        ("Docker", vec!["docker", "containerization"]),
        ("Rust", vec!["rust", "actix", "tokio"]),
        ("TypeScript", vec!["typescript", "tsconfig"]),
    ];

    for (name, matches) in tech_keywords {
        if matches.iter().any(|m| lower_body.contains(m)) {
            tech_stack.push(name.to_string());
        }
    }

    let mut emails: Vec<String> = emails_set.into_iter().collect();
    // Prioritize business prefix emails (sales@, info@, contact@)
    emails.sort_by_key(|e| {
        let prefix = e.split('@').next().unwrap_or("");
        if vec!["sales", "info", "contact", "support", "hello", "enquiries"].contains(&prefix) {
            0
        } else {
            1
        }
    });

    let mut phones: Vec<String> = phones_set.into_iter().collect();
    phones.sort();
    let mut internal_links: Vec<String> = internal_links_set.into_iter().collect();
    internal_links.truncate(20);
    let mut external_links: Vec<String> = external_links_set.into_iter().collect();
    external_links.truncate(30);

    ParsedContent {
        emails,
        phones,
        linkedin_url,
        contact_url,
        internal_links,
        external_links,
        hiring_signals,
        engineering_jobs,
        remote_jobs,
        outsourcing_keywords,
        tech_stack,
    }
}

fn is_valid_business_email(email: &str) -> bool {
    let lower = email.to_lowercase();
    if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".svg") || lower.ends_with(".js") || lower.ends_with(".css") {
        return false;
    }
    if lower.contains("example.com") || lower.contains("domain.com") || lower.contains("sentry.io") {
        return false;
    }
    true
}

pub fn extract_domain(url_str: &str) -> Option<String> {
    if let Ok(parsed) = Url::parse(url_str) {
        if let Some(host) = parsed.host_str() {
            let host_clean = host.trim_start_matches("www.");
            return Some(host_clean.to_lowercase());
        }
    }
    None
}
