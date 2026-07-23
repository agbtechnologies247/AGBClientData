use crate::models::{Company, LeadScoreBreakdown};

pub fn calculate_score(company: &mut Company, signals_found: &[String]) -> LeadScoreBreakdown {
    let mut hiring_bonus = 0;
    let mut engineering_jobs_score = 0;
    let mut remote_jobs_score = 0;
    let mut outsourcing_intent_score = 0;
    let mut contact_info_bonus = 0;

    if company.hiring {
        hiring_bonus += 20;
    }

    engineering_jobs_score += (company.engineering_jobs as i32 * 5).min(40);
    remote_jobs_score += (company.remote_jobs as i32 * 8).min(25);
    outsourcing_intent_score += (company.outsourcing_keywords as i32 * 25).min(80);

    if company.email.is_some() {
        contact_info_bonus += 20;
    }

    let base_score = 10;
    let total = base_score
        + hiring_bonus
        + engineering_jobs_score
        + remote_jobs_score
        + outsourcing_intent_score
        + contact_info_bonus;

    company.lead_score = total;
    company.priority_tier = match total {
        score if score >= 80 => "HIGH".to_string(),
        score if score >= 60 => "MEDIUM".to_string(),
        _ => "LOW".to_string(),
    };

    LeadScoreBreakdown {
        base_score,
        hiring_bonus,
        engineering_jobs_score,
        remote_jobs_score,
        outsourcing_intent_score,
        contact_info_bonus,
        total_score: total,
        signals_found: signals_found.to_vec(),
    }
}
