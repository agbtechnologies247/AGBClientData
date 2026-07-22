use regex::Regex;
use serde::{Deserialize, Serialize};
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub email_valid_syntax: bool,
    pub mx_record_found: bool,
    pub website_alive: bool,
    pub phone_e164: Option<String>,
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

    /// Level 2: DNS MX Record Lookup
    pub async fn verify_mx_record(&self, domain: &str) -> bool {
        match self.resolver.mx_lookup(domain).await {
            Ok(lookup) => !lookup.iter().collect::<Vec<_>>().is_empty(),
            Err(_) => false,
        }
    }

    /// Phone Normalization to E.164 format
    pub fn normalize_phone_e164(phone_raw: &str, country: &str) -> Option<String> {
        let digits: String = phone_raw.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 7 {
            return None;
        }

        if phone_raw.starts_with('+') {
            return Some(format!("+{}", digits));
        }

        if country == "UK" {
            if digits.starts_with('0') {
                return Some(format!("+44{}", &digits[1..]));
            }
            if digits.starts_with("44") {
                return Some(format!("+{}", digits));
            }
            return Some(format!("+44{}", digits));
        }

        // Default US / North America
        if digits.len() == 10 {
            return Some(format!("+1{}", digits));
        } else if digits.len() == 11 && digits.starts_with('1') {
            return Some(format!("+{}", digits));
        }

        Some(format!("+{}", digits))
    }

    /// Contact Confidence Score Calculation (0 - 100)
    pub async fn validate_contact_confidence(
        &self,
        email: Option<&str>,
        phone: Option<&str>,
        domain: &str,
        has_contact_page: bool,
        website_alive: bool,
    ) -> ValidationResult {
        let mut score = 0;
        let mut email_valid = false;
        let mut mx_found = false;

        if let Some(e) = email {
            if Self::validate_email_syntax(e) {
                email_valid = true;
                score += 10;

                // Extract domain from email for MX check
                if let Some(parts) = e.split('@').nth(1) {
                    if self.verify_mx_record(parts).await {
                        mx_found = true;
                        score += 20;
                    }
                }

                // Public business email bonus (info@, sales@, contact@)
                let prefix = e.split('@').next().unwrap_or("").to_lowercase();
                if vec!["info", "sales", "contact", "support", "hello", "enquiries", "partnerships"].contains(&prefix.as_str()) {
                    score += 30;
                } else {
                    score += 15;
                }
            }
        }

        if website_alive {
            score += 20;
        }

        if has_contact_page {
            score += 20;
        }

        let phone_e164 = phone.and_then(|p| Self::normalize_phone_e164(p, "US"));

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
            phone_e164,
            confidence_score,
            confidence_tier,
        }
    }
}
