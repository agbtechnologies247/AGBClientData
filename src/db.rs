use crate::campaign::{CreateCampaignRequest, EmailCampaign};
use crate::models::{Company, CrawlLogEntry, CrawlerStats, Investor, InvestorFilter, LeadFilter, Person, PersonFilter, ProxyInfo};
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let _ = conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -64000;
        ");

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        db.seed_data_if_empty()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS companies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                domain TEXT UNIQUE NOT NULL,
                website TEXT NOT NULL,
                country TEXT NOT NULL,
                city TEXT,
                industry TEXT,
                email TEXT,
                phone TEXT,
                contact_url TEXT,
                linkedin_url TEXT,
                hiring INTEGER NOT NULL DEFAULT 0,
                engineering_jobs INTEGER NOT NULL DEFAULT 0,
                remote_jobs INTEGER NOT NULL DEFAULT 0,
                outsourcing_keywords INTEGER NOT NULL DEFAULT 0,
                lead_score INTEGER NOT NULL DEFAULT 0,
                priority_tier TEXT NOT NULL DEFAULT 'LOW',
                tech_stack TEXT NOT NULL DEFAULT '[]',
                contact_person TEXT,
                last_crawled TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_companies_score ON companies(lead_score DESC);
            CREATE INDEX IF NOT EXISTS idx_companies_country ON companies(country);
            CREATE INDEX IF NOT EXISTS idx_companies_priority ON companies(priority_tier);

            CREATE TABLE IF NOT EXISTS crawled_domains (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL DEFAULT 'COMPLETED',
                last_crawled TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS search_queries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT UNIQUE NOT NULL,
                last_executed TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS people (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                company_id INTEGER NOT NULL DEFAULT 0,
                company_name TEXT NOT NULL,
                company_domain TEXT NOT NULL,
                name TEXT NOT NULL,
                title TEXT NOT NULL,
                normalized_role TEXT NOT NULL,
                decision_maker_score INTEGER NOT NULL DEFAULT 50,
                public_email TEXT,
                phone TEXT,
                linkedin_url TEXT,
                confidence_score INTEGER NOT NULL DEFAULT 80
            );

            CREATE INDEX IF NOT EXISTS idx_people_score ON people(decision_maker_score DESC);

            CREATE TABLE IF NOT EXISTS campaigns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                subject_template TEXT NOT NULL,
                body_template TEXT NOT NULL,
                target_role TEXT NOT NULL DEFAULT 'Technology Executive',
                status TEXT NOT NULL DEFAULT 'ACTIVE',
                total_recipients INTEGER NOT NULL DEFAULT 0,
                sent_count INTEGER NOT NULL DEFAULT 0,
                open_count INTEGER NOT NULL DEFAULT 0,
                click_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS outreach_emails (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                campaign_id INTEGER NOT NULL,
                recipient_email TEXT NOT NULL,
                recipient_name TEXT NOT NULL,
                recipient_title TEXT NOT NULL,
                company_name TEXT NOT NULL,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'QUEUED',
                sent_at TEXT
            );

            CREATE TABLE IF NOT EXISTS investors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                investor_type TEXT NOT NULL,
                website TEXT UNIQUE NOT NULL,
                country TEXT NOT NULL,
                city TEXT,
                focus TEXT NOT NULL DEFAULT '[]',
                stages TEXT NOT NULL DEFAULT '[]',
                check_size TEXT,
                public_email TEXT,
                phone TEXT,
                linkedin_url TEXT,
                portfolio_highlights TEXT NOT NULL DEFAULT '[]',
                recent_investments INTEGER NOT NULL DEFAULT 0,
                score INTEGER NOT NULL DEFAULT 0,
                priority_tier TEXT NOT NULL DEFAULT 'TIER 3',
                last_updated TEXT
            );

            CREATE TABLE IF NOT EXISTS proxies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT UNIQUE NOT NULL,
                protocol TEXT NOT NULL DEFAULT 'http',
                active INTEGER NOT NULL DEFAULT 1,
                success_count INTEGER NOT NULL DEFAULT 0,
                fail_count INTEGER NOT NULL DEFAULT 0,
                latency_ms INTEGER NOT NULL DEFAULT 0,
                last_used TEXT
            );

            CREATE TABLE IF NOT EXISTS crawl_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT UNIQUE NOT NULL,
                url TEXT NOT NULL,
                depth INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'PENDING'
            );

            CREATE TABLE IF NOT EXISTS crawl_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                domain TEXT NOT NULL,
                message TEXT NOT NULL
            );
            ",
        )?;

        let _ = conn.execute("ALTER TABLE companies ADD COLUMN contact_person TEXT", []);

        Ok(())
    }

    fn seed_data_if_empty(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let company_count: i64 = conn.query_row("SELECT COUNT(*) FROM companies", [], |r| r.get(0))?;

        if company_count == 0 {
            let seed_companies = vec![
                ("Apex Systems Solutions", "apexsystems.com", "https://apexsystems.com", "US", Some("New York"), Some("IT Services & Consulting"), Some("contact@apexsystems.com"), Some("+1 (212) 555-0192"), Some("https://apexsystems.com/contact"), Some("https://linkedin.com/company/apex-systems"), 1, 14, 8, 4, 95, "HIGH", "[\"React\", \"Node.js\", \"AWS\", \"Python\"]"),
                ("BlueSky Cloud Tech", "blueskytech.co.uk", "https://blueskytech.co.uk", "UK", Some("London"), Some("Cloud Migration & DevOps"), Some("sales@blueskytech.co.uk"), Some("+44 20 7946 0912"), Some("https://blueskytech.co.uk/get-in-touch"), Some("https://linkedin.com/company/blueskytech"), 1, 9, 6, 3, 85, "HIGH", "[\"Kubernetes\", \"Terraform\", \"Go\", \"Azure\"]"),
                ("Vanguard Digital Ops", "vanguard-digital.com", "https://vanguard-digital.com", "US", Some("Austin"), Some("Custom Software Development"), Some("info@vanguard-digital.com"), Some("+1 (512) 555-0843"), Some("https://vanguard-digital.com/contact"), Some("https://linkedin.com/company/vanguard-digital"), 1, 18, 12, 5, 115, "HIGH", "[\"Rust\", \"Java\", \"PostgreSQL\", \"Docker\"]"),
                ("Meridian IT Consultancy", "meridian-it.co.uk", "https://meridian-it.co.uk", "UK", Some("Manchester"), Some("Managed IT & Cybersecurity"), Some("hello@meridian-it.co.uk"), Some("+44 161 496 0184"), Some("https://meridian-it.co.uk/contact-us"), Some("+44 161 496 0184"), 1, 5, 2, 2, 68, "MEDIUM", "[\"Microsoft 365\", \"Cisco\", \"Python\"]"),
                ("Nexus Enterprise Labs", "nexuslabs.io", "https://nexuslabs.io", "US", Some("Boston"), Some("AI & Data Engineering"), Some("partnerships@nexuslabs.io"), Some("+1 (617) 555-0371"), Some("https://nexuslabs.io/contact"), Some("https://linkedin.com/company/nexuslabs-io"), 1, 11, 7, 3, 90, "HIGH", "[\"PyTorch\", \"TypeScript\", \"GCP\", \"Kafka\"]"),
                ("Beacon Digital Services", "beacondigital.co.uk", "https://beacondigital.co.uk", "UK", Some("Birmingham"), Some("Web & Mobile Apps"), Some("enquiries@beacondigital.co.uk"), Some("+44 121 496 0932"), Some("https://beacondigital.co.uk/contact"), Some("+44 121 496 0932"), 0, 2, 0, 1, 45, "LOW", "[\"Vue.js\", \"PHP\", \"MySQL\"]"),
                ("Crestline Software Corp", "crestlinesoft.com", "https://crestlinesoft.com", "US", Some("Chicago"), Some("Enterprise Resource Planning"), Some("contact@crestlinesoft.com"), Some("+1 (312) 555-0921"), Some("https://crestlinesoft.com/support"), Some("https://linkedin.com/company/crestlinesoft"), 1, 7, 4, 2, 72, "MEDIUM", "[\"C#\", \".NET Core\", \"SQL Server\"]"),
            ];

            let now = Utc::now().to_rfc3339();

            for c in seed_companies {
                conn.execute(
                    "INSERT INTO companies (name, domain, website, country, city, industry, email, phone, contact_url, linkedin_url, hiring, engineering_jobs, remote_jobs, outsourcing_keywords, lead_score, priority_tier, tech_stack, last_crawled)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                    params![c.0, c.1, c.2, c.3, c.4, c.5, c.6, c.7, c.8, c.9, c.10, c.11, c.12, c.13, c.14, c.15, c.16, now],
                )?;
                conn.execute(
                    "INSERT OR IGNORE INTO crawled_domains (domain, status, last_crawled) VALUES (?1, 'COMPLETED', ?2)",
                    params![c.1, now],
                )?;
            }

            // Seed proxies
            let sample_proxies = vec![
                ("http://185.199.229.156:8080", "http"),
                ("http://198.51.100.42:3128", "http"),
                ("socks5://192.0.2.71:1080", "socks5"),
            ];
            for p in sample_proxies {
                conn.execute(
                    "INSERT INTO proxies (url, protocol, active) VALUES (?1, ?2, 1)",
                    params![p.0, p.1],
                )?;
            }
        }

        // Seed People / Decision Makers
        let people_count: i64 = conn.query_row("SELECT COUNT(*) FROM people", [], |r| r.get(0))?;
        if people_count == 0 {
            let seed_people = vec![
                (1, "Apex Systems Solutions", "apexsystems.com", "David Miller", "Chief Technology Officer", "Technology Executive", 100, Some("cto@apexsystems.com"), Some("+1 (212) 555-0192"), Some("https://linkedin.com/in/david-miller-cto"), 95),
                (1, "Apex Systems Solutions", "apexsystems.com", "Sarah Jenkins", "VP of Engineering", "Engineering Leadership", 95, Some("s.jenkins@apexsystems.com"), Some("+1 (212) 555-0193"), Some("https://linkedin.com/in/sarah-jenkins-eng"), 92),
                (2, "BlueSky Cloud Tech", "blueskytech.co.uk", "Richard Taylor", "Director of Engineering", "Engineering Leadership", 92, Some("richard@blueskytech.co.uk"), Some("+44 20 7946 0912"), Some("https://linkedin.com/in/richard-taylor-uk"), 90),
                (3, "Vanguard Digital Ops", "vanguard-digital.com", "Michael Vance", "Founder & CEO", "Executive Management", 88, Some("mvance@vanguard-digital.com"), Some("+1 (512) 555-0843"), Some("https://linkedin.com/in/michael-vance-ceo"), 96),
                (5, "Nexus Enterprise Labs", "nexuslabs.io", "Dr. Elena Rostova", "Chief Information Officer", "Technology Executive", 75, Some("elena@nexuslabs.io"), Some("+1 (617) 555-0371"), Some("https://linkedin.com/in/elena-rostova-cio"), 88),
            ];

            for p in seed_people {
                conn.execute(
                    "INSERT INTO people (company_id, company_name, company_domain, name, title, normalized_role, decision_maker_score, public_email, phone, linkedin_url, confidence_score)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![p.0, p.1, p.2, p.3, p.4, p.5, p.6, p.7, p.8, p.9, p.10],
                )?;
            }
        }

        // Seed Default Campaign
        let campaign_count: i64 = conn.query_row("SELECT COUNT(*) FROM campaigns", [], |r| r.get(0))?;
        if campaign_count == 0 {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO campaigns (name, subject_template, body_template, target_role, status, total_recipients, sent_count, open_count, click_count, created_at)
                 VALUES ('Q3 CTO Engineering Outreach', 'Question regarding {{Company}} software roadmap', 'Hi {{First_Name}},\n\nI saw your role as {{Title}} at {{Company}}.\n\nWe at AGB Technologies help tech leaders scale engineering velocity.\n\nWould you be open to a 10-min chat?', 'Technology Executive', 'ACTIVE', 5, 5, 3, 2, ?1)",
                params![now],
            )?;
        }

        // Seed Investors
        let investor_count: i64 = conn.query_row("SELECT COUNT(*) FROM investors", [], |r| r.get(0))?;
        if investor_count == 0 {
            let seed_investors = vec![
                (
                    "SaaS Venture Partners", "Micro VC", "https://saasventures.com", "US", Some("San Francisco"),
                    "[\"B2B SaaS\", \"AI\", \"Enterprise Software\", \"Automation\", \"India\"]",
                    "[\"Pre-Seed\", \"Seed\"]", "$100K - $1M", Some("invest@saasventures.com"), Some("+1 (415) 555-0144"),
                    Some("https://linkedin.com/company/saas-ventures"), "[\"UiPath\", \"Freshworks\", \"Postman\"]", 8, 145, "TIER 1"
                ),
                (
                    "Indus Tech Syndicate", "Syndicate", "https://industechsyndicate.io", "US", Some("New York"),
                    "[\"B2B SaaS\", \"Offshore IT\", \"India\", \"DevOps\", \"Cloud\"]",
                    "[\"Seed\", \"Series A\"]", "$250K - $2M", Some("deals@industechsyndicate.io"), Some("+1 (212) 555-0812"),
                    Some("https://linkedin.com/company/indus-tech"), "[\"BrowserStack\", \"Hasura\", \"Chargebee\"]", 12, 160, "TIER 1"
                ),
                (
                    "Frontier SaaS Angels", "Angel Group", "https://frontiersaas.org", "UK", Some("London"),
                    "[\"B2B SaaS\", \"Enterprise\", \"Automation\", \"AI\"]",
                    "[\"Pre-Seed\", \"Seed\"]", "$50K - $300K", Some("pitch@frontiersaas.org"), Some("+44 20 7946 0882"),
                    Some("https://linkedin.com/company/frontier-saas"), "[\"Revolut\", \"HopIn\", \"Snyk\"]", 6, 120, "TIER 1"
                ),
                (
                    "Apex Capital Family Office", "Family Office", "https://apexcapfamily.com", "US", Some("Boston"),
                    "[\"Enterprise Software\", \"ERP\", \"Cloud Services\", \"India\"]",
                    "[\"Seed\", \"Series A\"]", "$500K - $3M", Some("investments@apexcapfamily.com"), Some("+1 (617) 555-0941"),
                    None, "[\"Snowflake\", \"Datadog\"]", 4, 110, "TIER 2"
                ),
                (
                    "Global SaaS Accelerator", "Accelerator", "https://globalsaas.vc", "UK", Some("Manchester"),
                    "[\"B2B SaaS\", \"AI\", \"DevOps\"]",
                    "[\"Pre-Seed\"]", "$100K - $250K", Some("apply@globalsaas.vc"), Some("+44 161 496 0321"),
                    Some("https://linkedin.com/company/global-saas-acc"), "[\"GitLab\", \"Vercel\"]", 15, 135, "TIER 1"
                ),
            ];

            let now = Utc::now().to_rfc3339();
            for inv in seed_investors {
                conn.execute(
                    "INSERT INTO investors (name, investor_type, website, country, city, focus, stages, check_size, public_email, phone, linkedin_url, portfolio_highlights, recent_investments, score, priority_tier, last_updated)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    params![inv.0, inv.1, inv.2, inv.3, inv.4, inv.5, inv.6, inv.7, inv.8, inv.9, inv.10, inv.11, inv.12, inv.13, inv.14, now],
                )?;
            }

            conn.execute(
                "INSERT INTO crawl_logs (timestamp, level, domain, message) VALUES (?1, ?2, ?3, ?4)",
                params![now, "INFO", "PEOPLE", "Decision Maker Intelligence database seeded with CTO, VP Eng, and Founder contacts."],
            )?;
        }

        Ok(())
    }

    pub fn get_campaigns(&self) -> Result<Vec<EmailCampaign>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, subject_template, body_template, target_role, status, total_recipients, sent_count, open_count, click_count, created_at FROM campaigns ORDER BY id DESC")?;
        let c_iter = stmt.query_map([], |r| {
            Ok(EmailCampaign {
                id: r.get(0)?,
                name: r.get(1)?,
                subject_template: r.get(2)?,
                body_template: r.get(3)?,
                target_role: r.get(4)?,
                status: r.get(5)?,
                total_recipients: r.get(6)?,
                sent_count: r.get(7)?,
                open_count: r.get(8)?,
                click_count: r.get(9)?,
                created_at: r.get(10)?,
            })
        })?;

        let mut campaigns = Vec::new();
        for c in c_iter {
            campaigns.push(c?);
        }
        Ok(campaigns)
    }

    pub fn create_campaign(&self, req: &CreateCampaignRequest) -> Result<EmailCampaign> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO campaigns (name, subject_template, body_template, target_role, status, total_recipients, sent_count, open_count, click_count, created_at)
             VALUES (?1, ?2, ?3, ?4, 'ACTIVE', 0, 0, 0, 0, ?5)",
            params![req.name, req.subject_template, req.body_template, req.target_role, now],
        )?;

        let id = conn.last_insert_rowid();

        Ok(EmailCampaign {
            id,
            name: req.name.clone(),
            subject_template: req.subject_template.clone(),
            body_template: req.body_template.clone(),
            target_role: req.target_role.clone(),
            status: "ACTIVE".to_string(),
            total_recipients: 0,
            sent_count: 0,
            open_count: 0,
            click_count: 0,
            created_at: now,
        })
    }

    pub fn is_domain_crawled(&self, domain: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM crawled_domains WHERE domain = ?1",
            params![domain],
            |r| r.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn mark_domain_crawled(&self, domain: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO crawled_domains (domain, status, last_crawled) VALUES (?1, ?2, ?3)
             ON CONFLICT(domain) DO UPDATE SET status=excluded.status, last_crawled=excluded.last_crawled",
            params![domain, status, now],
        )?;
        Ok(())
    }

    pub fn is_query_executed(&self, query: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM search_queries WHERE query = ?1",
            params![query],
            |r| r.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn mark_query_executed(&self, query: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO search_queries (query, last_executed) VALUES (?1, ?2)
             ON CONFLICT(query) DO UPDATE SET last_executed=excluded.last_executed",
            params![query, now],
        )?;
        Ok(())
    }

    pub fn get_stats(&self) -> Result<CrawlerStats> {
        let conn = self.conn.lock().unwrap();

        let total_companies: usize = conn.query_row("SELECT COUNT(*) FROM companies", [], |r| r.get(0))?;
        let high_intent_leads: usize = conn.query_row("SELECT COUNT(*) FROM companies WHERE priority_tier = 'HIGH'", [], |r| r.get(0))?;
        let medium_intent_leads: usize = conn.query_row("SELECT COUNT(*) FROM companies WHERE priority_tier = 'MEDIUM'", [], |r| r.get(0))?;
        let hiring_companies: usize = conn.query_row("SELECT COUNT(*) FROM companies WHERE hiring = 1", [], |r| r.get(0))?;
        let leads_with_email: usize = conn.query_row("SELECT COUNT(*) FROM companies WHERE email IS NOT NULL AND email != ''", [], |r| r.get(0))?;
        let leads_with_phone: usize = conn.query_row("SELECT COUNT(*) FROM companies WHERE phone IS NOT NULL AND phone != ''", [], |r| r.get(0))?;
        let total_decision_makers: usize = conn.query_row("SELECT COUNT(*) FROM people", [], |r| r.get(0))?;
        let total_investors: usize = conn.query_row("SELECT COUNT(*) FROM investors", [], |r| r.get(0))?;
        let tier1_investors: usize = conn.query_row("SELECT COUNT(*) FROM investors WHERE priority_tier = 'TIER 1'", [], |r| r.get(0))?;
        let total_crawled_pages: usize = conn.query_row("SELECT COUNT(*) FROM crawled_domains", [], |r| r.get(0)).unwrap_or(42);
        let active_proxies: usize = conn.query_row("SELECT COUNT(*) FROM proxies WHERE active = 1", [], |r| r.get(0))?;

        Ok(CrawlerStats {
            total_companies,
            high_intent_leads,
            medium_intent_leads,
            hiring_companies,
            leads_with_email,
            leads_with_phone,
            total_decision_makers,
            total_investors,
            tier1_investors,
            total_crawled_pages,
            active_proxies,
            crawler_status: "IDLE".to_string(),
            current_domain: None,
        })
    }

    pub fn get_leads(&self, filter: &LeadFilter) -> Result<(Vec<Company>, usize)> {
        let conn = self.conn.lock().unwrap();

        let mut query = String::from("SELECT id, name, domain, website, country, city, industry, email, phone, contact_url, linkedin_url, hiring, engineering_jobs, remote_jobs, outsourcing_keywords, lead_score, priority_tier, tech_stack, contact_person, last_crawled FROM companies WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(min) = filter.min_score {
            query.push_str(" AND lead_score >= ?");
            params_vec.push(Box::new(min));
        }

        if let Some(ref country) = filter.country {
            if !country.is_empty() && country != "ALL" {
                query.push_str(" AND country = ?");
                params_vec.push(Box::new(country.clone()));
            }
        }

        if let Some(hiring) = filter.hiring_only {
            if hiring {
                query.push_str(" AND hiring = 1");
            }
        }

        if let Some(has_email) = filter.has_email {
            if has_email {
                query.push_str(" AND email IS NOT NULL AND email != ''");
            }
        }

        if let Some(ref tier) = filter.priority {
            if !tier.is_empty() && tier != "ALL" {
                query.push_str(" AND priority_tier = ?");
                params_vec.push(Box::new(tier.clone()));
            }
        }

        if let Some(ref q) = filter.search_query {
            if !q.trim().is_empty() {
                let pattern = format!("%{}%", q.trim());
                query.push_str(" AND (name LIKE ? OR domain LIKE ? OR industry LIKE ? OR tech_stack LIKE ? OR contact_person LIKE ?)");
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern));
            }
        }

        let count_sql = format!("SELECT COUNT(*) FROM ({})", query);
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let total_count: usize = conn.query_row(&count_sql, params_refs.as_slice(), |r| r.get(0))?;

        query.push_str(" ORDER BY lead_score DESC, id DESC LIMIT ? OFFSET ?");
        params_vec.push(Box::new(filter.limit as i64));
        let offset = ((filter.page.max(1) - 1) * filter.limit) as i64;
        params_vec.push(Box::new(offset));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query)?;
        
        let company_iter = stmt.query_map(params_refs.as_slice(), |row| {
            let tech_stack_str: String = row.get(17)?;
            let tech_stack: Vec<String> = serde_json::from_str(&tech_stack_str).unwrap_or_default();
            Ok(Company {
                id: row.get(0)?,
                name: row.get(1)?,
                domain: row.get(2)?,
                website: row.get(3)?,
                country: row.get(4)?,
                city: row.get(5)?,
                industry: row.get(6)?,
                email: row.get(7)?,
                phone: row.get(8)?,
                contact_url: row.get(9)?,
                linkedin_url: row.get(10)?,
                hiring: row.get::<_, i32>(11)? == 1,
                engineering_jobs: row.get(12)?,
                remote_jobs: row.get(13)?,
                outsourcing_keywords: row.get(14)?,
                lead_score: row.get(15)?,
                priority_tier: row.get(16)?,
                tech_stack,
                contact_person: row.get(18)?,
                last_crawled: row.get(19)?,
            })
        })?;

        let mut leads = Vec::new();
        for company in company_iter {
            leads.push(company?);
        }

        Ok((leads, total_count))
    }

    pub fn save_company(&self, c: &Company) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tech_stack_json = serde_json::to_string(&c.tech_stack).unwrap_or_default();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO companies (name, domain, website, country, city, industry, email, phone, contact_url, linkedin_url, hiring, engineering_jobs, remote_jobs, outsourcing_keywords, lead_score, priority_tier, tech_stack, contact_person, last_crawled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
             ON CONFLICT(domain) DO UPDATE SET
                name=excluded.name, website=excluded.website, country=excluded.country, city=excluded.city,
                industry=excluded.industry, email=COALESCE(excluded.email, companies.email),
                phone=COALESCE(excluded.phone, companies.phone), contact_url=COALESCE(excluded.contact_url, companies.contact_url),
                linkedin_url=COALESCE(excluded.linkedin_url, companies.linkedin_url), hiring=excluded.hiring,
                engineering_jobs=excluded.engineering_jobs, remote_jobs=excluded.remote_jobs,
                outsourcing_keywords=excluded.outsourcing_keywords, lead_score=excluded.lead_score,
                priority_tier=excluded.priority_tier, tech_stack=excluded.tech_stack,
                contact_person=COALESCE(excluded.contact_person, companies.contact_person), last_crawled=excluded.last_crawled",
            params![
                c.name, c.domain, c.website, c.country, c.city, c.industry, c.email, c.phone,
                c.contact_url, c.linkedin_url, c.hiring as i32, c.engineering_jobs, c.remote_jobs,
                c.outsourcing_keywords, c.lead_score, c.priority_tier, tech_stack_json, c.contact_person, now
            ],
        )?;

        Ok(())
    }

    pub fn get_people(&self, filter: &PersonFilter) -> Result<(Vec<Person>, usize)> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from("SELECT id, company_id, company_name, company_domain, name, title, normalized_role, decision_maker_score, public_email, phone, linkedin_url, confidence_score FROM people WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(min) = filter.min_score {
            query.push_str(" AND decision_maker_score >= ?");
            params_vec.push(Box::new(min));
        }

        if let Some(ref role) = filter.role {
            if !role.is_empty() && role != "ALL" {
                query.push_str(" AND normalized_role = ?");
                params_vec.push(Box::new(role.clone()));
            }
        }

        if let Some(ref dom) = filter.domain {
            if !dom.is_empty() {
                query.push_str(" AND company_domain = ?");
                params_vec.push(Box::new(dom.clone()));
            }
        }

        if let Some(ref q) = filter.search_query {
            if !q.trim().is_empty() {
                let pattern = format!("%{}%", q.trim());
                query.push_str(" AND (name LIKE ? OR title LIKE ? OR company_name LIKE ?)");
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern));
            }
        }

        let count_sql = format!("SELECT COUNT(*) FROM ({})", query);
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let total_count: usize = conn.query_row(&count_sql, params_refs.as_slice(), |r| r.get(0))?;

        query.push_str(" ORDER BY decision_maker_score DESC, id DESC LIMIT ? OFFSET ?");
        params_vec.push(Box::new(filter.limit as i64));
        let offset = ((filter.page.max(1) - 1) * filter.limit) as i64;
        params_vec.push(Box::new(offset));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query)?;
        
        let p_iter = stmt.query_map(params_refs.as_slice(), |r| {
            Ok(Person {
                id: r.get(0)?,
                company_id: r.get(1)?,
                company_name: r.get(2)?,
                company_domain: r.get(3)?,
                name: r.get(4)?,
                title: r.get(5)?,
                normalized_role: r.get(6)?,
                decision_maker_score: r.get(7)?,
                public_email: r.get(8)?,
                phone: r.get(9)?,
                linkedin_url: r.get(10)?,
                confidence_score: r.get(11)?,
            })
        })?;

        let mut people = Vec::new();
        for p in p_iter {
            people.push(p?);
        }

        Ok((people, total_count))
    }

    pub fn save_person(&self, p: &Person) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO people (company_id, company_name, company_domain, name, title, normalized_role, decision_maker_score, public_email, phone, linkedin_url, confidence_score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                p.company_id, p.company_name, p.company_domain, p.name, p.title, p.normalized_role,
                p.decision_maker_score, p.public_email, p.phone, p.linkedin_url, p.confidence_score
            ],
        )?;
        Ok(())
    }

    pub fn get_investors(&self, filter: &InvestorFilter) -> Result<(Vec<Investor>, usize)> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from("SELECT id, name, investor_type, website, country, city, focus, stages, check_size, public_email, phone, linkedin_url, portfolio_highlights, recent_investments, score, priority_tier, last_updated FROM investors WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(min) = filter.min_score {
            query.push_str(" AND score >= ?");
            params_vec.push(Box::new(min));
        }

        if let Some(ref country) = filter.country {
            if !country.is_empty() && country != "ALL" {
                query.push_str(" AND country = ?");
                params_vec.push(Box::new(country.clone()));
            }
        }

        if let Some(ref itype) = filter.investor_type {
            if !itype.is_empty() && itype != "ALL" {
                query.push_str(" AND investor_type = ?");
                params_vec.push(Box::new(itype.clone()));
            }
        }

        if let Some(ref focus) = filter.focus {
            if !focus.is_empty() && focus != "ALL" {
                query.push_str(" AND focus LIKE ?");
                params_vec.push(Box::new(format!("%{}%", focus)));
            }
        }

        if let Some(has_email) = filter.has_email {
            if has_email {
                query.push_str(" AND public_email IS NOT NULL AND public_email != ''");
            }
        }

        if let Some(ref q) = filter.search_query {
            if !q.trim().is_empty() {
                let pattern = format!("%{}%", q.trim());
                query.push_str(" AND (name LIKE ? OR focus LIKE ? OR portfolio_highlights LIKE ?)");
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern));
            }
        }

        let count_sql = format!("SELECT COUNT(*) FROM ({})", query);
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let total_count: usize = conn.query_row(&count_sql, params_refs.as_slice(), |r| r.get(0))?;

        query.push_str(" ORDER BY score DESC, id DESC LIMIT ? OFFSET ?");
        params_vec.push(Box::new(filter.limit as i64));
        let offset = ((filter.page.max(1) - 1) * filter.limit) as i64;
        params_vec.push(Box::new(offset));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query)?;
        
        let inv_iter = stmt.query_map(params_refs.as_slice(), |r| {
            let focus_str: String = r.get(6)?;
            let stages_str: String = r.get(7)?;
            let port_str: String = r.get(12)?;

            let focus: Vec<String> = serde_json::from_str(&focus_str).unwrap_or_default();
            let stages: Vec<String> = serde_json::from_str(&stages_str).unwrap_or_default();
            let portfolio_highlights: Vec<String> = serde_json::from_str(&port_str).unwrap_or_default();

            Ok(Investor {
                id: r.get(0)?,
                name: r.get(1)?,
                investor_type: r.get(2)?,
                website: r.get(3)?,
                country: r.get(4)?,
                city: r.get(5)?,
                focus,
                stages,
                check_size: r.get(8)?,
                public_email: r.get(9)?,
                phone: r.get(10)?,
                linkedin_url: r.get(11)?,
                portfolio_highlights,
                recent_investments: r.get(13)?,
                score: r.get(14)?,
                priority_tier: r.get(15)?,
                last_updated: r.get(16)?,
            })
        })?;

        let mut investors = Vec::new();
        for inv in inv_iter {
            investors.push(inv?);
        }

        Ok((investors, total_count))
    }

    pub fn save_investor(&self, inv: &Investor) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let focus_json = serde_json::to_string(&inv.focus).unwrap_or_default();
        let stages_json = serde_json::to_string(&inv.stages).unwrap_or_default();
        let port_json = serde_json::to_string(&inv.portfolio_highlights).unwrap_or_default();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO investors (name, investor_type, website, country, city, focus, stages, check_size, public_email, phone, linkedin_url, portfolio_highlights, recent_investments, score, priority_tier, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(website) DO UPDATE SET
                name=excluded.name, investor_type=excluded.investor_type, country=excluded.country, city=excluded.city,
                focus=excluded.focus, stages=excluded.stages, check_size=excluded.check_size,
                public_email=COALESCE(excluded.public_email, investors.public_email),
                phone=COALESCE(excluded.phone, investors.phone), linkedin_url=COALESCE(excluded.linkedin_url, investors.linkedin_url),
                portfolio_highlights=excluded.portfolio_highlights, recent_investments=excluded.recent_investments,
                score=excluded.score, priority_tier=excluded.priority_tier, last_updated=excluded.last_updated",
            params![
                inv.name, inv.investor_type, inv.website, inv.country, inv.city, focus_json, stages_json,
                inv.check_size, inv.public_email, inv.phone, inv.linkedin_url, port_json,
                inv.recent_investments, inv.score, inv.priority_tier, now
            ],
        )?;

        Ok(())
    }

    pub fn add_proxy(&self, url: &str, protocol: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO proxies (url, protocol, active) VALUES (?1, ?2, 1)",
            params![url, protocol],
        )?;
        Ok(())
    }

    pub fn get_proxies(&self) -> Result<Vec<ProxyInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, url, protocol, active, success_count, fail_count, latency_ms, last_used FROM proxies ORDER BY id DESC")?;
        let proxy_iter = stmt.query_map([], |r| {
            Ok(ProxyInfo {
                id: r.get(0)?,
                url: r.get(1)?,
                protocol: r.get(2)?,
                active: r.get::<_, i32>(3)? == 1,
                success_count: r.get(4)?,
                fail_count: r.get(5)?,
                latency_ms: r.get(6)?,
                last_used: r.get(7)?,
            })
        })?;

        let mut proxies = Vec::new();
        for p in proxy_iter {
            proxies.push(p?);
        }
        Ok(proxies)
    }

    pub fn log_event(&self, level: &str, domain: &str, message: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO crawl_logs (timestamp, level, domain, message) VALUES (?1, ?2, ?3, ?4)",
            params![now, level, domain, message],
        )?;
        Ok(())
    }

    pub fn get_recent_logs(&self, limit: usize) -> Result<Vec<CrawlLogEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, timestamp, level, domain, message FROM crawl_logs ORDER BY id DESC LIMIT ?")?;
        let log_iter = stmt.query_map(params![limit as i64], |r| {
            Ok(CrawlLogEntry {
                id: r.get(0)?,
                timestamp: r.get(1)?,
                level: r.get(2)?,
                domain: r.get(3)?,
                message: r.get(4)?,
            })
        })?;

        let mut logs = Vec::new();
        for l in log_iter {
            logs.push(l?);
        }
        Ok(logs)
    }

    pub fn clear_executed_queries(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM search_queries", [])?;
        Ok(deleted)
    }

    pub fn enqueue_domain(&self, domain: &str, url: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM crawled_domains WHERE domain = ?1",
            params![domain],
            |r| r.get(0),
        )?;
        if count > 0 {
            return Ok(false);
        }

        let inserted = conn.execute(
            "INSERT OR IGNORE INTO crawl_queue (domain, url, depth, status) VALUES (?1, ?2, 0, 'PENDING')",
            params![domain, url],
        )?;
        Ok(inserted > 0)
    }

    pub fn pop_pending_queue_domains(&self, limit: usize) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain, url FROM crawl_queue 
             WHERE status = 'PENDING' 
             AND domain NOT IN (SELECT domain FROM crawled_domains) 
             LIMIT ?"
        )?;
        
        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })?;

        let mut ids = Vec::new();
        let mut urls = Vec::new();

        for row in rows {
            if let Ok((id, _domain, url)) = row {
                ids.push(id);
                urls.push(url);
            }
        }

        for id in ids {
            let _ = conn.execute("UPDATE crawl_queue SET status = 'PROCESSING' WHERE id = ?1", params![id]);
        }

        Ok(urls)
    }
}

