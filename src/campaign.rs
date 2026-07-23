use crate::db::Database;
use crate::models::Person;
use crate::validator::ContactValidator;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCampaign {
    pub id: i64,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub target_role: String,
    pub status: String, // "DRAFT", "ACTIVE", "COMPLETED"
    pub total_recipients: usize,
    pub sent_count: usize,
    pub open_count: usize,
    pub click_count: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutreachEmail {
    pub id: i64,
    pub campaign_id: i64,
    pub recipient_email: String,
    pub recipient_name: String,
    pub recipient_title: String,
    pub company_name: String,
    pub subject: String,
    pub body: String,
    pub status: String, // "QUEUED", "SENT", "BOUNCED", "REPLIED", "FAILED"
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub target_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostingerSmtpConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub sender_email: String,
    pub sender_name: String,
    pub auth_password: String,
}

impl Default for HostingerSmtpConfig {
    fn default() -> Self {
        Self {
            smtp_host: "smtp.hostinger.com".to_string(),
            smtp_port: 465,
            sender_email: "support@agbtechnologies.com".to_string(),
            sender_name: "Bhramit Pardhi [Shubham]".to_string(),
            auth_password: "Bhramit@143".to_string(),
        }
    }
}

pub struct CampaignEngine;

impl CampaignEngine {
    pub fn default_alfred_billsoft_template() -> (String, String) {
        let subject = "Enterprise Operations & Business Management Platform Demo - AGB Technologies".to_string();
        let body = r#"Dear Sir/Madam,

I hope this email finds you well.

My name is Bhramit Pardhi [Shubham], Founder of AGB Technologies.

We have built and successfully deployed A.L.F.R.E.D. (Autonomous Lifecycle Framework for Resilient Enterprise Digitalization), an AI-powered enterprise operations platform that helps organizations automate IT operations, managed services, infrastructure management, user lifecycle, compliance, monitoring, and business workflows from a single intelligent platform.

Live Production Demo:
https://alfred.agbtechnologies.in/

In addition, we have developed BillSoft, a modern cloud-based business management and billing platform suitable for SMEs and growing businesses.

Live Production Demo:
https://billsoft.agbtechnologies.com/

We are currently looking to connect with organizations that may be interested in:

* Becoming an early customer and leveraging our platform to reduce operational costs.
* Exploring strategic partnerships or white-label opportunities.
* Investing in or acquiring our enterprise software portfolio.

If you're looking to modernize your operations, streamline business processes, or reduce infrastructure and support costs, we'd be delighted to demonstrate how A.L.F.R.E.D. and BillSoft can help your organization.

Please let us know if you would like to become one of our customers. We would be happy to show you how A.L.F.R.E.D. can significantly reduce your operational costs while improving efficiency and visibility across your organization.

I would be happy to share our pitch deck, product architecture, business model, financial projections, and provide demo account access upon request.

Thank you for your time and consideration. I look forward to the opportunity to connect.

Kind regards,

Bhramit Pardhi [Shubham]
Founder, AGB Technologies

Email: agbtechnologies247@gmail.com
Phone: +91 9049874780"#.to_string();

        (subject, body)
    }

    pub fn personalize_template(
        subject_tmpl: &str,
        body_tmpl: &str,
        person_name: &str,
        company_name: &str,
        title: &str,
    ) -> (String, String) {
        let first_name = person_name.split_whitespace().next().unwrap_or(person_name);

        let subject = subject_tmpl
            .replace("{{First_Name}}", first_name)
            .replace("{{Company}}", company_name)
            .replace("{{Title}}", title);

        let body = body_tmpl
            .replace("{{Name}}", person_name)
            .replace("{{First_Name}}", first_name)
            .replace("{{Company}}", company_name)
            .replace("{{Title}}", title);

        (subject, body)
    }

    /// Sends a batch of unsent email leads using Hostinger SMTP with strict deduplication
    pub async fn dispatch_outreach_batch(db: &Database, limit: usize) -> Result<usize, String> {
        let unsent_leads = db.get_unsent_leads_batch(limit).map_err(|e| e.to_string())?;

        if unsent_leads.is_empty() {
            info!("No unsent verified leads available for outreach batch.");
            return Ok(0);
        }

        let config = HostingerSmtpConfig::default();
        let creds = Credentials::new(config.sender_email.clone(), config.auth_password.clone());

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            match AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host) {
                Ok(builder) => builder
                    .credentials(creds)
                    .port(config.smtp_port)
                    .build(),
                Err(e) => return Err(format!("Failed to build SMTP relay: {}", e)),
            };

        let (default_subject, default_body) = Self::default_alfred_billsoft_template();
        let mut sent_count = 0;

        let validator = ContactValidator::new();

        for company in unsent_leads {
            let email_addr = match company.email.clone() {
                Some(e) if !e.trim().is_empty() => e.trim().to_string(),
                _ => continue,
            };

            // Double check deduplication: Skip if recipient email was already sent
            if db.is_email_already_sent(&email_addr).unwrap_or(false) {
                let _ = db.log_event("WARN", &company.domain, &format!("Skipping duplicate email address: {}", email_addr));
                continue;
            }

            // Level 1 & 2 Validation: Validate syntax and MX DNS record before sending
            if !ContactValidator::validate_email_syntax(&email_addr) {
                let _ = db.log_event("WARN", &company.domain, &format!("Skipping invalid email syntax: {}", email_addr));
                let _ = db.record_sent_email_history(&email_addr, &company.name, "INVALID");
                continue;
            }

            if let Some(domain_part) = email_addr.split('@').nth(1) {
                if !validator.verify_mx_record(domain_part).await {
                    let _ = db.log_event("WARN", &company.domain, &format!("Skipping domain with missing MX DNS record: {}", domain_part));
                    let _ = db.record_sent_email_history(&email_addr, &company.name, "BOUNCED");
                    continue;
                }
            }

            let person_name = company.contact_person.clone().unwrap_or_else(|| "Technology Executive".to_string());
            let title = company.contact_position.clone().unwrap_or_else(|| "CTO / VP Engineering".to_string());

            let (subject, body) = Self::personalize_template(
                &default_subject,
                &default_body,
                &person_name,
                &company.name,
                &title,
            );

            let from_header = format!("{} <{}>", config.sender_name, config.sender_email);

            let email = match Message::builder()
                .from(match from_header.parse() {
                    Ok(f) => f,
                    Err(_) => match config.sender_email.parse() {
                        Ok(f) => f,
                        Err(e) => {
                            let _ = db.log_event("ERROR", &company.domain, &format!("Invalid sender email format: {}", e));
                            continue;
                        }
                    },
                })
                .to(match email_addr.parse() {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = db.log_event("WARN", &company.domain, &format!("Invalid recipient email format: {}", e));
                        let _ = db.record_sent_email_history(&email_addr, &company.name, "INVALID");
                        continue;
                    }
                })
                .subject(&subject)
                .body(body) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = db.log_event("ERROR", &company.domain, &format!("Email message build error: {}", e));
                        continue;
                    }
                };

            match mailer.send(email).await {
                Ok(_) => {
                    sent_count += 1;
                    let _ = db.record_sent_email_history(&email_addr, &company.name, "SENT");
                    let _ = db.log_event("SUCCESS", &company.domain, &format!("Outreach email successfully sent via Hostinger SMTP to {}", email_addr));
                }
                Err(e) => {
                    let _ = db.record_sent_email_history(&email_addr, &company.name, "FAILED");
                    let _ = db.log_event("ERROR", &company.domain, &format!("Hostinger SMTP send error to {}: {}", email_addr, e));
                }
            }

            // Brief delay between email dispatches to prevent rate-limit throttling
            sleep(Duration::from_millis(1500)).await;
        }

        Ok(sent_count)
    }

    /// Spawns autonomous 24/7 hourly cron background loop
    pub fn start_hourly_outreach_daemon(db: Database) {
        tokio::spawn(async move {
            let _ = db.log_event("INFO", "OUTREACH_DAEMON", "Hostinger Hourly Email Outreach Cron Daemon activated (Interval: 1 Hour).");

            loop {
                info!("Triggering autonomous hourly email outreach batch via Hostinger SMTP...");
                match Self::dispatch_outreach_batch(&db, 15).await {
                    Ok(count) => {
                        let _ = db.log_event("INFO", "OUTREACH_DAEMON", &format!("Hourly outreach batch complete: {} emails dispatched.", count));
                    }
                    Err(err) => {
                        let _ = db.log_event("ERROR", "OUTREACH_DAEMON", &format!("Outreach batch dispatch error: {}", err));
                    }
                }

                // Sleep for 1 hour (3600 seconds) before next execution
                sleep(Duration::from_secs(3600)).await;
            }
        });
    }
}
