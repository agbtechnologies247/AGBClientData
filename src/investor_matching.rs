use crate::models::{Investor, InvestorMatchRequest, InvestorMatchResult, InvestorMatcherTrait};

pub struct DefaultInvestorMatcher;

impl DefaultInvestorMatcher {
    pub fn new() -> Self {
        Self
    }
}

impl InvestorMatcherTrait for DefaultInvestorMatcher {
    fn match_investor(&self, investor: &Investor, request: &InvestorMatchRequest) -> InvestorMatchResult {
        match_investor(investor, request)
    }
}

pub fn match_investor(inv: &Investor, req: &InvestorMatchRequest) -> InvestorMatchResult {
    let mut score = 20; // Base score
    let mut reasons = Vec::new();

    // 1. Sector & Focus Matching
    let inv_focus_lower: Vec<String> = inv.focus.iter().map(|f| f.to_lowercase()).collect();

    for sec in &req.sectors {
        let sec_lower = sec.to_lowercase();
        if inv_focus_lower.iter().any(|f| f.contains(&sec_lower) || sec_lower.contains(f)) {
            let pts = match sec_lower.as_str() {
                "b2b saas" | "saas" => 40,
                "ai" | "artificial intelligence" => 25,
                "enterprise software" | "enterprise" => 20,
                "automation" => 20,
                _ => 15,
            };
            score += pts;
            reasons.push(format!("Invests in {}", sec));
        }
    }

    // 2. Geographic & Target Market Matching
    let inv_country_lower = inv.country.to_lowercase();
    for market in &req.target_market {
        let m_lower = market.to_lowercase();
        if inv_focus_lower.iter().any(|f| f.contains(&m_lower)) || inv_country_lower == m_lower {
            score += 20;
            reasons.push(format!("Target Market Match: {}", market));
            break;
        }
    }

    // 3. Funding Stage Matching
    let inv_stages_lower: Vec<String> = inv.stages.iter().map(|s| s.to_lowercase()).collect();
    let req_stage_lower = req.funding_stage.to_lowercase();

    if inv_stages_lower.iter().any(|s| s.contains(&req_stage_lower) || req_stage_lower.contains(s)) {
        let pts = match req_stage_lower.as_str() {
            "seed" => 30,
            "pre-seed" => 25,
            "series a" => 20,
            _ => 15,
        };
        score += pts;
        reasons.push(format!("Active in {} Stage", req.funding_stage));
    }

    // 4. Recent Activity Boost
    if inv.recent_investments > 5 {
        score += 15;
        reasons.push(format!("Highly Active ({} recent investments)", inv.recent_investments));
    }

    // Cap match score between 0 and 100
    let match_score = score.clamp(0, 100);

    InvestorMatchResult {
        investor: inv.clone(),
        match_score,
        match_reasons: reasons,
    }
}
