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

    pub fn default_investor_template() -> (String, String) {
        let subject = "Strategic Investment & B2B SaaS Partnership Opportunity - AGB Technologies".to_string();
        let body = r#"Dear {{Name}},

I hope this email finds you well.

My name is Bhramit Pardhi [Shubham], Founder of AGB Technologies.

We are currently building and expanding our enterprise SaaS portfolio, featuring:

1. A.L.F.R.E.D. (Autonomous Lifecycle Framework for Resilient Enterprise Digitalization) - AI-powered IT operations and workflow automation platform.
Live Production Demo: https://alfred.agbtechnologies.in/

2. BillSoft - Modern cloud-based SME business management and billing platform.
Live Production Demo: https://billsoft.agbtechnologies.com/

Given your active investment focus in {{Title}}, we would be delighted to share our pitch deck, product roadmap, and discuss strategic investment or partnership opportunities with {{Company}}.

I would be happy to arrange a brief call or share access to our investor data room.

Kind regards,

Bhramit Pardhi [Shubham]
Founder, AGB Technologies
Email: agbtechnologies247@gmail.com
Phone: +91 9049874780"#.to_string();

        (subject, body)
    }

    /// Sends a batch of unsent email leads using Hostinger SMTP with high-throughput worker pool
    pub async fn dispatch_outreach_batch(db: &Database, limit: usize) -> Result<usize, String> {
        let mut total_sent = 0;

        // 1. Dispatch to Ranked Decision Makers & Executives (People)
        if let Ok(count) = Self::dispatch_people_outreach_batch(db, limit).await {
            total_sent += count;
        }

        // 2. Dispatch to Verified B2B SaaS & AI Investors
        if let Ok(count) = Self::dispatch_investor_outreach_batch(db, limit).await {
            total_sent += count;
        }

        // 3. Fallback/Complementary: Dispatch to Verified Companies
        if total_sent < limit {
            if let Ok(count) = Self::dispatch_company_outreach_batch(db, limit - total_sent).await {
                total_sent += count;
            }
        }

        Ok(total_sent)
    }

    pub async fn dispatch_people_outreach_batch(db: &Database, limit: usize) -> Result<usize, String> {
        let unsent_people = db.get_unsent_people_batch(limit).map_err(|e| e.to_string())?;
        if unsent_people.is_empty() {
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

        for person in unsent_people {
            let email_addr = match person.public_email.clone() {
                Some(e) if !e.trim().is_empty() => e.trim().to_string(),
                _ => continue,
            };

            if db.is_email_already_sent(&email_addr).unwrap_or(false) {
                continue;
            }

            if !ContactValidator::validate_email_syntax(&email_addr) {
                let _ = db.record_sent_email_history(&email_addr, &person.company_name, "INVALID");
                continue;
            }

            if let Some(domain_part) = email_addr.split('@').nth(1) {
                if !validator.verify_mx_record(domain_part).await {
                    let _ = db.record_sent_email_history(&email_addr, &person.company_name, "BOUNCED");
                    continue;
                }
            }

            let (subject, body) = Self::personalize_template(
                &default_subject,
                &default_body,
                &person.name,
                &person.company_name,
                &person.title,
            );

            let from_header = format!("{} <{}>", config.sender_name, config.sender_email);

            let email = match Message::builder()
                .from(match from_header.parse() {
                    Ok(f) => f,
                    Err(_) => match config.sender_email.parse() {
                        Ok(f) => f,
                        Err(_) => continue,
                    },
                })
                .reply_to(match "support@agbtechnologies.com".parse() {
                    Ok(r) => r,
                    Err(_) => match config.sender_email.parse() {
                        Ok(r) => r,
                        Err(_) => continue,
                    },
                })
                .to(match email_addr.parse() {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = db.record_sent_email_history(&email_addr, &person.company_name, "INVALID");
                        continue;
                    }
                })
                .subject(&subject)
                .body(body) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

            match mailer.send(email).await {
                Ok(_) => {
                    sent_count += 1;
                    let _ = db.record_sent_email_history(&email_addr, &person.company_name, "SENT");
                    let _ = db.log_event("SUCCESS", &person.company_domain, &format!("Executive outreach email sent to {} ({}, {})", email_addr, person.name, person.title));
                }
                Err(e) => {
                    let _ = db.record_sent_email_history(&email_addr, &person.company_name, "FAILED");
                    let _ = db.log_event("ERROR", &person.company_domain, &format!("SMTP error to {}: {}", email_addr, e));
                }
            }

            sleep(Duration::from_millis(500)).await;
        }

        Ok(sent_count)
    }

    pub async fn dispatch_investor_outreach_batch(db: &Database, limit: usize) -> Result<usize, String> {
        let unsent_investors = db.get_unsent_investors_batch(limit).map_err(|e| e.to_string())?;
        if unsent_investors.is_empty() {
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

        let (default_subject, default_body) = Self::default_investor_template();
        let mut sent_count = 0;
        let validator = ContactValidator::new();

        for inv in unsent_investors {
            let email_addr = match inv.public_email.clone() {
                Some(e) if !e.trim().is_empty() => e.trim().to_string(),
                _ => continue,
            };

            if db.is_email_already_sent(&email_addr).unwrap_or(false) {
                continue;
            }

            if !ContactValidator::validate_email_syntax(&email_addr) {
                let _ = db.record_sent_email_history(&email_addr, &inv.name, "INVALID");
                continue;
            }

            if let Some(domain_part) = email_addr.split('@').nth(1) {
                if !validator.verify_mx_record(domain_part).await {
                    let _ = db.record_sent_email_history(&email_addr, &inv.name, "BOUNCED");
                    continue;
                }
            }

            let focus_summary = if inv.focus.is_empty() { "B2B SaaS & AI".to_string() } else { inv.focus.join(", ") };

            let (subject, body) = Self::personalize_template(
                &default_subject,
                &default_body,
                &inv.name,
                &inv.name,
                &focus_summary,
            );

            let from_header = format!("{} <{}>", config.sender_name, config.sender_email);

            let email = match Message::builder()
                .from(match from_header.parse() {
                    Ok(f) => f,
                    Err(_) => match config.sender_email.parse() {
                        Ok(f) => f,
                        Err(e) => continue,
                    },
                })
                .to(match email_addr.parse() {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = db.record_sent_email_history(&email_addr, &inv.name, "INVALID");
                        continue;
                    }
                })
                .subject(&subject)
                .body(body) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

            match mailer.send(email).await {
                Ok(_) => {
                    sent_count += 1;
                    let _ = db.record_sent_email_history(&email_addr, &inv.name, "SENT");
                    let _ = db.log_event("SUCCESS", &inv.website, &format!("Investor outreach email sent to {} ({}, {})", email_addr, inv.name, inv.investor_type));
                }
                Err(e) => {
                    let _ = db.record_sent_email_history(&email_addr, &inv.name, "FAILED");
                    let _ = db.log_event("ERROR", &inv.website, &format!("SMTP error to {}: {}", email_addr, e));
                }
            }

            sleep(Duration::from_millis(500)).await;
        }

        Ok(sent_count)
    }

    pub async fn dispatch_company_outreach_batch(db: &Database, limit: usize) -> Result<usize, String> {
        let unsent_leads = db.get_unsent_leads_batch(limit).map_err(|e| e.to_string())?;

        if unsent_leads.is_empty() {
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

            if db.is_email_already_sent(&email_addr).unwrap_or(false) {
                continue;
            }

            if !ContactValidator::validate_email_syntax(&email_addr) {
                let _ = db.record_sent_email_history(&email_addr, &company.name, "INVALID");
                continue;
            }

            if let Some(domain_part) = email_addr.split('@').nth(1) {
                if !validator.verify_mx_record(domain_part).await {
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
                        Err(e) => continue,
                    },
                })
                .to(match email_addr.parse() {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = db.record_sent_email_history(&email_addr, &company.name, "INVALID");
                        continue;
                    }
                })
                .subject(&subject)
                .body(body) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

            match mailer.send(email).await {
                Ok(_) => {
                    sent_count += 1;
                    let _ = db.record_sent_email_history(&email_addr, &company.name, "SENT");
                    let _ = db.log_event("SUCCESS", &company.domain, &format!("Company outreach email sent to {}", email_addr));
                }
                Err(e) => {
                    let _ = db.record_sent_email_history(&email_addr, &company.name, "FAILED");
                    let _ = db.log_event("ERROR", &company.domain, &format!("SMTP error to {}: {}", email_addr, e));
                }
            }

            sleep(Duration::from_millis(500)).await;
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
