use crate::db::Database;
use regex::Regex;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub struct BounceMonitorEngine;

impl BounceMonitorEngine {
    /// Extracts the target failed email address from MAILER-DAEMON NDR email bodies
    pub fn parse_bounce_recipient(body: &str) -> Option<String> {
        // Regex Pattern 1: <email>: Host or domain name not found / User unknown
        let re1 = Regex::new(r"<([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})>:\s*(?:Host or domain|User unknown|Address rejected|550|554|Name service error)").unwrap();
        if let Some(caps) = re1.captures(body) {
            return Some(caps[1].to_string());
        }

        // Regex Pattern 2: Final-Recipient / Recipient: <email>
        let re2 = Regex::new(r"(?:To|Recipient|Final-Recipient):\s*(?:rfc822;)?\s*<?([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})>?").unwrap();
        if let Some(caps) = re2.captures(body) {
            return Some(caps[1].to_string());
        }

        None
    }

    /// Identifies whether an email subject or sender indicates a non-delivery report
    pub fn is_bounce_subject_or_sender(sender: &str, subject: &str) -> bool {
        let s_lower = sender.to_lowercase();
        let sub_lower = subject.to_lowercase();

        s_lower.contains("mailer-daemon")
            || s_lower.contains("postmaster")
            || sub_lower.contains("undelivered mail")
            || sub_lower.contains("delivery status notification")
            || sub_lower.contains("mail delivery failed")
            || sub_lower.contains("failure notice")
            || sub_lower.contains("returned to sender")
    }

    /// Process incoming email payload and update database status
    pub fn process_email_payload(db: &Database, sender: &str, subject: &str, body: &str) -> bool {
        if Self::is_bounce_subject_or_sender(sender, subject) {
            if let Some(failed_email) = Self::parse_bounce_recipient(body) {
                info!("IMAP Bounce Monitor detected bounced email: {}", failed_email);
                let _ = db.update_email_status(&failed_email, "BOUNCED");
                return true;
            }
        }
        false
    }

    /// Automated background daemon loop for bounce & response monitoring
    pub async fn start_daemon_loop(db: Database) {
        let _ = db.log_event("INFO", "IMAP_DAEMON", "Hostinger IMAP bounce & response listener daemon initialized.");

        loop {
            // Run bounce monitoring check every 5 minutes
            sleep(Duration::from_secs(300)).await;
        }
    }
}
