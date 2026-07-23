use crate::models::EmailValidatorTrait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub email_valid_syntax: bool,
    pub mx_record_found: bool,
    pub website_alive: bool,
    pub confidence_score: i32, // 0 - 100
    pub confidence_tier: String, // "EXCELLENT", "GOOD", "AVERAGE", "POOR"
}

pub struct ContactValidator {
    resolver: TokioAsyncResolver,
}

impl ContactValidator {
    pub fn new() -> Self {
        let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        Self { resolver }
    }

    /// Level 1: Email Syntax Validation
    pub fn validate_email_syntax(email: &str) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(email) {
            return false;
        }

        // Filter out disposable email domains
        let lower = email.to_lowercase();
        let disposable = vec!["tempmail", "mailinator", "guerrillamail", "10minutemail", "trashmail"];
        if disposable.iter().any(|d| lower.contains(d)) {
            return false;
        }

        true
    }

    /// Advanced Email ML Confidence Scoring Engine
    pub fn calculate_email_ml_score(
        email: &str,
        page_context: &str,
        mx_record_found: bool,
        is_catchall: bool,
    ) -> (i32, String, bool) {
        if !Self::validate_email_syntax(email) {
            return (-100, "LOW".to_string(), false);
        }

        if !mx_record_found {
            return (-100, "LOW".to_string(), false);
        }

        let mut score = 20; // +20 MX Record Exists

        let lower_email = email.to_lowercase();
        let parts: Vec<&str> = lower_email.split('@').collect();
        if parts.len() != 2 {
            return (-100, "LOW".to_string(), false);
        }

        let prefix = parts[0];
        let domain = parts[1];

        // Generic Prefix Penalties
        let generic_penalties = vec![
            ("info", -10),
            ("contact", -10),
            ("support", -25),
            ("help", -25),
            ("sales", -5),
            ("admin", -30),
            ("billing", -25),
            ("accounts", -25),
            ("careers", -30),
            ("jobs", -30),
            ("noreply", -100),
            ("no-reply", -100),
            ("donotreply", -100),
            ("webmaster", -30),
        ];

        let mut is_generic = false;
        for (g_prefix, penalty) in generic_penalties {
            if prefix == g_prefix || prefix.starts_with(g_prefix) {
                score += penalty;
                is_generic = true;
                break;
            }
        }

        // Person-Based Email Bonus (+40)
        let name_email_regex = Regex::new(r"^[a-z]+[._-][a-z]+$|^[a-z]{3,15}$").unwrap();
        if !is_generic && name_email_regex.is_match(prefix) {
            score += 40;
        }

        // Domain Reputation
        let free_providers = vec!["gmail.com", "yahoo.com", "outlook.com", "hotmail.com", "aol.com", "icloud.com"];
        if free_providers.contains(&domain) {
            score -= 10;
        } else {
            score += 15;
        }

        // Page Context Analysis
        let ctx_lower = page_context.to_lowercase();
        if ctx_lower.contains("about") || ctx_lower.contains("team") || ctx_lower.contains("leadership") || ctx_lower.contains("management") || ctx_lower.contains("founders") {
            score += 20;
        } else if ctx_lower.contains("contact") {
            score += 10;
        } else if ctx_lower.contains("footer") || ctx_lower.contains("terms") || ctx_lower.contains("privacy") {
            score -= 15;
        }

        if is_catchall {
            score -= 15;
        }

        let clamped_score = score.clamp(0, 100);
        let confidence_tier = match clamped_score {
            s if s >= 90 => "EXCELLENT".to_string(),
            s if s >= 70 => "GOOD".to_string(),
            s if s >= 50 => "MEDIUM".to_string(),
            _ => "LOW".to_string(),
        };

        (clamped_score, confidence_tier, clamped_score >= 50)
    }

    /// Level 2: DNS MX Record Lookup
    pub async fn verify_mx_record(&self, domain: &str) -> bool {
        match self.resolver.mx_lookup(domain).await {
            Ok(lookup) => !lookup.iter().collect::<Vec<_>>().is_empty(),
            Err(_) => false,
        }
    }

    /// Contact Confidence Score Calculation (0 - 100)
    pub async fn validate_contact_confidence(
        &self,
        email: Option<&str>,
        _domain: &str,
        has_contact_page: bool,
        website_alive: bool,
    ) -> ValidationResult {
        let mut score = 0;
        let mut email_valid = false;
        let mut mx_found = false;

        if let Some(e) = email {
            if Self::validate_email_syntax(e) {
                email_valid = true;
                score += 20;

                if let Some(parts) = e.split('@').nth(1) {
                    if self.verify_mx_record(parts).await {
                        mx_found = true;
                        score += 30;
                    }
                }

                let prefix = e.split('@').next().unwrap_or("").to_lowercase();
                if vec!["info", "sales", "contact", "support", "hello", "enquiries", "partnerships"].contains(&prefix.as_str()) {
                    score += 30;
                } else {
                    score += 20;
                }
            }
        }

        if website_alive {
            score += 10;
        }

        if has_contact_page {
            score += 10;
        }

        let confidence_score = score.clamp(0, 100);
        let confidence_tier = match confidence_score {
            s if s >= 90 => "EXCELLENT".to_string(),
            s if s >= 70 => "GOOD".to_string(),
            s if s >= 50 => "AVERAGE".to_string(),
            _ => "POOR".to_string(),
        };

        ValidationResult {
            email_valid_syntax: email_valid,
            mx_record_found: mx_found,
            website_alive,
            confidence_score,
            confidence_tier,
        }
    }
}

impl EmailValidatorTrait for ContactValidator {
    fn validate_email(&self, email: &str) -> bool {
        Self::validate_email_syntax(email)
    }
}
