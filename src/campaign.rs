use crate::models::Person;
use serde::{Deserialize, Serialize};

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
    pub status: String, // "QUEUED", "SENT", "FAILED"
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub target_role: String,
}

pub struct CampaignEngine;

impl CampaignEngine {
    pub fn personalize_template(
        subject_tmpl: &str,
        body_tmpl: &str,
        person: &Person,
        sender_company: &str,
    ) -> (String, String) {
        let subject = subject_tmpl
            .replace("{{First_Name}}", person.name.split_whitespace().next().unwrap_or(&person.name))
            .replace("{{Company}}", &person.company_name)
            .replace("{{Title}}", &person.title);

        let body = body_tmpl
            .replace("{{Name}}", &person.name)
            .replace("{{First_Name}}", person.name.split_whitespace().next().unwrap_or(&person.name))
            .replace("{{Company}}", &person.company_name)
            .replace("{{Title}}", &person.title)
            .replace("{{Sender_Company}}", sender_company);

        (subject, body)
    }

    pub fn default_cto_outreach_template() -> (String, String) {
        let subject = "Question regarding {{Company}}'s software engineering roadmap".to_string();
        let body = "Hi {{First_Name}},\n\nI was looking at {{Company}}'s technology leadership and saw your role as {{Title}}.\n\nWe at {{Sender_Company}} help high-growth tech companies scale their dedicated engineering teams and cloud infrastructure seamlessly.\n\nWould you be open to a quick 10-minute chat this Thursday to discuss your technical priorities for this quarter?\n\nBest regards,\nAGB Technologies Team".to_string();
        (subject, body)
    }
}
