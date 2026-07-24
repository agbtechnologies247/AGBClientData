#[cfg(test)]
mod tests {
    use crate::investor_matching::match_investor;
    use crate::models::{Company, Investor, InvestorMatchRequest};
    use crate::parser::parse_html;
    use crate::people::DecisionMakerEngine;
    use crate::score::calculate_score;

    #[test]
    fn test_parser_email_extraction() {
        let html = r#"
            <html>
                <body>
                    <p>Contact our engineering team at contact@acmesoftware.com or support@acmesoftware.com</p>
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
            contact_url: Some("https://testcorp.com/contact".to_string()),
            linkedin_url: Some("https://linkedin.com/company/testcorp".to_string()),
            hiring: true,
            engineering_jobs: 5,
            remote_jobs: 3,
            outsourcing_keywords: 2,
            lead_score: 0,
            priority_tier: "LOW".to_string(),
            tech_stack: vec!["React".to_string(), "Node.js".to_string()],
            contact_person: Some("Alex Rivera".to_string()),
            contact_position: Some("Chief Technology Officer".to_string()),
            qualification_stage: "DISCOVERED".to_string(),
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

    #[test]
    fn test_database_clear_and_reseeding() {
        use crate::db::Database;
        let db = Database::new(":memory:").expect("Failed to create in-memory database");
        let stats_before = db.get_stats().expect("Failed to get stats before clear");

        assert!(stats_before.total_companies > 0, "Initial companies count should be > 0");
        assert!(stats_before.total_decision_makers > 0, "Initial decision makers count should be > 0");
        assert!(stats_before.total_investors > 0, "Initial investors count should be > 0");

        db.clear_all_data().expect("Failed to clear database");
        let stats_after = db.get_stats().expect("Failed to get stats after clear");

        assert!(stats_after.total_companies > 0, "Companies count after clear & re-seed should be > 0");
        assert!(stats_after.total_decision_makers > 0, "Decision makers count after clear & re-seed should be > 0");
        assert!(stats_after.total_investors > 0, "Investors count after clear & re-seed should be > 0");
    }

    #[test]
    fn test_email_deduplication_guard() {
        use crate::db::Database;
        let db = Database::new(":memory:").expect("Failed to create in-memory database");

        let test_email = "cto@apexsystems.com";
        assert!(!db.is_email_already_sent(test_email).unwrap(), "Email should not be sent initially");

        db.record_sent_email_history(test_email, "Apex Systems Solutions", "SENT").unwrap();
        assert!(db.is_email_already_sent(test_email).unwrap(), "Email should be recorded as sent");

        let (history, total) = db.get_sent_emails_history(Some("SENT"), 1, 10).unwrap();
        assert_eq!(total, 1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].1, test_email);
        assert_eq!(history[0].3, "SENT");
    }

    #[test]
    fn test_alfred_billsoft_template_personalization() {
        use crate::campaign::CampaignEngine;
        let (subject, body) = CampaignEngine::default_alfred_billsoft_template();

        assert!(subject.contains("Enterprise Operations"));
        assert!(body.contains("A.L.F.R.E.D."));
        assert!(body.contains("BillSoft"));
        assert!(body.contains("Bhramit Pardhi [Shubham]"));
        assert!(body.contains("support@agbtechnologies.com"));
        assert!(body.contains("+91 9049874780"));

        let (personalized_sub, personalized_body) = CampaignEngine::personalize_template(
            &subject,
            &body,
            "David Miller",
            "Apex Systems Solutions",
            "Chief Technology Officer",
        );

        assert_eq!(personalized_sub, subject);
        assert!(personalized_body.contains("Bhramit Pardhi [Shubham]"));
    }

    #[tokio::test]
    async fn test_live_crawler_and_auto_queue_discovery() {
        use crate::db::Database;
        use crate::proxy::ProxyManager;
        use crate::crawler::AntiBlockingCrawler;

        let db = Database::new(":memory:").expect("Failed to create in-memory db");
        let proxy_mgr = ProxyManager::new(vec![]);
        let crawler = AntiBlockingCrawler::new(db.clone(), proxy_mgr);

        let initial_stats = db.get_stats().unwrap();
        
        let seeds = vec![
            "https://thoughtworks.com".to_string(),
            "https://epam.com".to_string(),
        ];

        crawler.start_crawl(seeds, Some("fast".to_string())).await;
        
        // Wait up to 10 seconds for async crawl to complete
        for _ in 0..20 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if !crawler.is_running() {
                break;
            }
        }

        let updated_stats = db.get_stats().unwrap();
        assert!(updated_stats.total_companies >= initial_stats.total_companies);
        assert!(updated_stats.total_decision_makers >= initial_stats.total_decision_makers);
    }

    #[test]
    fn test_linkedin_url_validation() {
        use crate::validator::ContactValidator;

        assert!(!ContactValidator::is_valid_linkedin_url("https://www.linkedin.com/company/"));
        assert!(!ContactValidator::is_valid_linkedin_url("https://www.linkedin.com/in/"));
        assert!(!ContactValidator::is_valid_linkedin_url("https://www.linkedin.com/404/"));
        assert!(!ContactValidator::is_valid_linkedin_url("https://www.linkedin.com/shareArticle"));

        assert!(ContactValidator::is_valid_linkedin_url("https://www.linkedin.com/in/alex-rivera-tech"));
        assert!(ContactValidator::is_valid_linkedin_url("https://www.linkedin.com/company/thoughtworks"));
    }

    #[test]
    fn test_bounce_signature_extraction() {
        use crate::bounce_monitor::BounceMonitorEngine;
        use crate::db::Database;

        let db = Database::new(":memory:").unwrap();
        db.record_sent_email_history("contact@strataai.tech", "Strata AI", "SENT").unwrap();

        let sample_ndr_body = r#"
            The mail system
            <contact@strataai.tech>: Host or domain name not found. Name service error for name=strataai.tech type=A: Host not found
        "#;

        let parsed = BounceMonitorEngine::parse_bounce_recipient(sample_ndr_body);
        assert_eq!(parsed, Some("contact@strataai.tech".to_string()));

        let processed = BounceMonitorEngine::process_email_payload(
            &db,
            "MAILER-DAEMON@mailchannels.net",
            "Undelivered Mail Returned to Sender",
            sample_ndr_body,
        );

        assert!(processed);

        let (history, _) = db.get_sent_emails_history(Some("BOUNCED"), 1, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].1, "contact@strataai.tech");
        assert_eq!(history[0].3, "BOUNCED");
    }

    #[test]
    fn test_open_and_click_tracking() {
        use crate::db::Database;

        let db = Database::new(":memory:").unwrap();
        let history_id = db.record_sent_email_history("exec@acme.com", "Acme Inc", "SENT").unwrap();
        assert!(history_id > 0);

        let opened = db.record_email_open_by_id(history_id).unwrap();
        assert!(opened);

        let (history, _) = db.get_sent_emails_history(None, 1, 10).unwrap();
        assert_eq!(history[0].3, "OPENED");

        let clicked = db.record_email_click_by_id(history_id).unwrap();
        assert!(clicked);

        let (history2, _) = db.get_sent_emails_history(None, 1, 10).unwrap();
        assert_eq!(history2[0].3, "CLICKED");
    }

    #[test]
    fn test_tech_stack_signature_fingerprinting() {
        use crate::parser::parse_html;

        let sample_html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script src="https://cdn.shopify.com/s/files/theme.js"></script>
                <script src="https://googletagmanager.com/gtag/js?id=UA-12345"></script>
                <script src="https://js.stripe.com/v3/"></script>
                <link rel="stylesheet" href="/_next/static/css/styles.css" />
            </head>
            <body>
                <a href="mailto:contact@acme.com">Contact Sales</a>
            </body>
            </html>
        "#;

        let parsed = parse_html("https://acme.com", sample_html);
        assert!(parsed.tech_stack.contains(&"React".to_string()));
        assert!(parsed.tech_stack.contains(&"Next.js".to_string()));
        assert!(parsed.tech_stack.contains(&"Shopify".to_string()));
        assert!(parsed.tech_stack.contains(&"Stripe".to_string()));
        assert!(parsed.tech_stack.contains(&"Google Analytics".to_string()));
    }
}
