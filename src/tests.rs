#[cfg(test)]
mod tests {
    use crate::investor_matching::match_investor;
    use crate::models::{Company, Investor, InvestorMatchRequest};
    use crate::parser::parse_html;
    use crate::people::DecisionMakerEngine;
    use crate::score::calculate_score;
    use crate::validator::ContactValidator;

    #[test]
    fn test_parser_email_and_phone_extraction() {
        let html = r#"
            <html>
                <body>
                    <p>Contact our engineering team at contact@acmesoftware.com or support@acmesoftware.com</p>
                    <p>Call US Sales: +1 (212) 555-0199 or UK Office: +44 20 7946 0912</p>
                    <a href="https://linkedin.com/company/acme-software">LinkedIn Profile</a>
                    <p>We're hiring React, Node.js, and Python engineers for remote team positions!</p>
                </body>
            </html>
        "#;

        let parsed = parse_html("https://acmesoftware.com", html);

        assert!(parsed.emails.contains(&"contact@acmesoftware.com".to_string()));
        assert!(parsed.emails.contains(&"support@acmesoftware.com".to_string()));
        assert!(parsed.hiring_signals.iter().any(|s| s.contains("hiring")));
        assert!(parsed.tech_stack.contains(&"React".to_string()));
        assert!(parsed.tech_stack.contains(&"Python".to_string()));
    }

    #[test]
    fn test_decision_maker_title_classification() {
        let (score1, role1) = DecisionMakerEngine::classify_title("Chief Technology Officer");
        assert_eq!(score1, 100);
        assert_eq!(role1, "Technology Executive");

        let (score2, role2) = DecisionMakerEngine::classify_title("VP of Engineering");
        assert_eq!(score2, 95);
        assert_eq!(role2, "Engineering Leadership");

        let (score3, role3) = DecisionMakerEngine::classify_title("Founder & CEO");
        assert_eq!(score3, 88);
        assert_eq!(role3, "Executive Management");
    }

    #[test]
    fn test_lead_score_calculation() {
        let mut company = Company {
            id: 1,
            name: "Test Corp".to_string(),
            domain: "testcorp.com".to_string(),
            website: "https://testcorp.com".to_string(),
            country: "US".to_string(),
            city: None,
            industry: Some("Software".to_string()),
            email: Some("contact@testcorp.com".to_string()),
            phone: Some("+12125550199".to_string()),
            contact_url: Some("https://testcorp.com/contact".to_string()),
            linkedin_url: Some("https://linkedin.com/company/testcorp".to_string()),
            hiring: true,
            engineering_jobs: 5,
            remote_jobs: 3,
            outsourcing_keywords: 2,
            lead_score: 0,
            priority_tier: "LOW".to_string(),
            tech_stack: vec!["React".to_string(), "Node.js".to_string()],
            last_crawled: None,
        };

        let signals = vec!["hiring".to_string(), "software engineer".to_string()];
        calculate_score(&mut company, &signals);

        assert!(company.lead_score >= 80);
        assert_eq!(company.priority_tier, "HIGH");
    }

    #[test]
    fn test_investor_matching_engine() {
        let investor = Investor {
            id: 1,
            name: "SaaS Capital".to_string(),
            investor_type: "Micro VC".to_string(),
            website: "https://saascapital.com".to_string(),
            country: "US".to_string(),
            city: Some("San Francisco".to_string()),
            focus: vec!["B2B SaaS".to_string(), "AI".to_string(), "India".to_string()],
            stages: vec!["Seed".to_string()],
            check_size: Some("$250K - $1M".to_string()),
            public_email: Some("invest@saascapital.com".to_string()),
            phone: None,
            linkedin_url: None,
            portfolio_highlights: vec!["Freshworks".to_string()],
            recent_investments: 5,
            score: 150,
            priority_tier: "TIER 1".to_string(),
            last_updated: None,
        };

        let req = InvestorMatchRequest {
            company_name: "AGB Technologies".to_string(),
            sectors: vec!["B2B SaaS".to_string(), "AI".to_string()],
            target_market: vec!["India".to_string()],
            funding_stage: "Seed".to_string(),
            funding_amount: Some("$250K - $1M".to_string()),
        };

        let match_res = match_investor(&investor, &req);
        assert!(match_res.match_score >= 85);
        assert!(match_res.match_reasons.iter().any(|r| r.contains("B2B SaaS")));
    }
}
