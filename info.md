Today 10:52 AM
I want to build a script which gives available IT and service companies in US AND UK . 

I want their numbers and emails to send my product details and call them 

Yes. You can build this, but I recommend collecting only publicly available business contact information (company phone numbers, sales emails, contact forms, websites). Avoid scraping personal emails or bypassing website restrictions. That keeps your process much safer and more compliant with laws like GDPR, CAN-SPAM, and UK PECR.

A good pipeline would be:

Find companies
Google Maps
Clutch
GoodFirms
Crunchbase (public pages)
Yellow Pages
Company websites
UK Companies House (public company information)
Extract
Company Name
Website
Country
City
Services
General Email (info@, sales@, contact@)
Phone Number
Contact Page URL
LinkedIn URL
Visit each website
Crawl only a few pages:
/
/contact
/about
/team
Extract emails using regex.
Extract phone numbers.
Ignore duplicates.

Store in PostgreSQL or CSV

Company
Website
Email
Phone
Country
Industry
Source
Last Crawled
Send outreach
Use SMTP or services like Amazon SES, SendGrid, or Mailgun.
Personalize emails.
Respect unsubscribe requests and applicable email marketing rules.

Since you're already using Node.js, a suitable stack is:

Playwright (dynamic websites)
Cheerio (HTML parsing)
Axios (HTTP requests)
BullMQ (job queue)
PostgreSQL
Redis
Nodemailer

You can also enrich the data using business lead platforms that provide company emails and phone numbers from public sources.

Given your SaaS experience, I'd build it as a scalable pipeline:

Scraper Service
Queue Service
Email Verification Service
Lead Database
Campaign Service
Analytics Dashboard

This can comfortably collect hundreds of thousands of company records from the US and UK over time while respecting website terms and applicable regulations.

Yes, but the key point is this:

There is no public filter that says "this US/UK company is actively looking to outsource to Indian IT companies." You have to infer that from multiple signals.

The highest-quality leads are companies showing outsourcing intent or rapid engineering growth.

A lead-scoring system might look like this:

Signal	Score
Hiring Software Engineers	+20
Hiring Engineering Managers	+15
Hiring Remote Developers	+25
Hiring Contractors	+30
Looking for Offshore Teams	+80
"Development Partner" mentioned	+80
Startup with <200 employees	+25
Recently funded	+40
Growing engineering team	+35
Already works with India	+70
The Excel could contain columns like:
Company	Website	Country	Industry	Employees	Hiring	Engineering Roles	Remote	IT Outsourcing Mention	Email	Phone	Contact Page	LinkedIn	Lead Score

Then you filter for:

Lead Score > 70
Hiring = Yes
Engineering Roles > 5
Employees < 1000
Public business contact available
Additional enrichment

You can also collect:

Current open positions
Tech stack
Number of engineering jobs
Remote-first status
Founded year
Revenue estimate (if publicly available)
Funding stage (for startups)
ATS (Greenhouse, Lever, Ashby, Workday, etc.)
Engineering director or procurement contact page (if publicly listed)
How to identify good prospects

The strongest signals include:

Companies advertising multiple software engineering openings.
Organizations hiring remote developers across time zones.
Businesses explicitly seeking contractors, implementation partners, or consulting services.
Fast-growing startups or scale-ups expanding engineering teams.
Companies that publicly mention distributed teams or existing offshore development partnerships.
Output format

The final Excel workbook could include:

High Priority (Lead Score ≥ 80)
Medium Priority (60–79)
Low Priority (<60)
Raw Data (all collected records)
Summary Dashboard (country, industry, score distribution, hiring trends)

This would give your sales team a prioritized outreach list rather than just a directory of IT companies.

One important consideration: some data sources (such as LinkedIn job postings, Crunchbase, or commercial directories) have terms that restrict automated scraping. If you build this system, it's better to use public company websites, official job pages, and APIs or licensed data where available, and to contact companies through their publicly listed business contact channels.

How do you think you will do this in Rust.. Show me example code

For this kind of project, I'd avoid trying to scrape everything from Google or LinkedIn directly. Instead, I'd build a distributed crawler that starts from legitimate public sources (company websites, public directories where permitted, official careers pages) and enriches the data with hiring signals.

A clean architecture in Rust could look like this:

                 Seed URLs
                       │
      ┌────────────────┴────────────────┐
      │                                 │
Company Discovery              Company Queue
      │                                 │
      └──────────────┬──────────────────┘
                     │
             Rust Async Workers
                     │
     ┌───────────────┼────────────────┐
     │               │                │
 HTML Fetcher   Contact Parser   Career Parser
     │               │                │
     └───────────────┼────────────────┘
                     │
              Company Object
                     │
             Lead Score Engine
                     │
                PostgreSQL
                     │
             Excel Export API
Project Structure
lead-crawler/

src/

    main.rs

    config.rs

    crawler.rs

    parser.rs

    careers.rs

    contacts.rs

    score.rs

    database.rs

    exporter.rs

Cargo.toml
Company Model
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct Company {

    pub name: String,

    pub website: String,

    pub country: String,

    pub email: Option<String>,

    pub phone: Option<String>,

    pub hiring: bool,

    pub engineering_jobs: usize,

    pub remote_jobs: usize,

    pub outsourcing_keywords: usize,

    pub lead_score: i32,
}
Async HTTP Fetch
use reqwest::Client;

pub async fn fetch(url: &str) -> Result<String, reqwest::Error> {

    let client = Client::builder()

        .user_agent("AGB Lead Bot")

        .build()?;

    let body = client

        .get(url)

        .send()

        .await?

        .text()

        .await?;

    Ok(body)
}
Email Extraction
use regex::Regex;

pub fn emails(html: &str) -> Vec<String> {

    let regex = Regex::new(

        r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}"

    ).unwrap();

    regex.find_iter(html)

        .map(|m| m.as_str().to_string())

        .collect()
}
Phone Extraction
let regex = Regex::new(

r"(\+\d{1,3})?[- ]?\(?\d+\)?[- ]?\d+[- ]?\d+"

).unwrap();
Hiring Detection
const WORDS: &[&str] = &[
    "careers",
    "jobs",
    "join our team",
    "we're hiring",
    "software engineer",
    "developer",
    "engineering manager",
    "full stack",
    "backend",
    "frontend",
];
pub fn hiring(html: &str) -> bool {

    let lower = html.to_lowercase();

    WORDS.iter().any(|w| lower.contains(w))
}
Lead Score
pub fn score(company: &mut Company) {

    let mut score = 0;

    if company.hiring {
        score += 20;
    }

    score += company.engineering_jobs as i32 * 5;

    score += company.remote_jobs as i32 * 8;

    score += company.outsourcing_keywords as i32 * 25;

    company.lead_score = score;
}
Concurrent Crawl
use futures::stream::{self, StreamExt};

let websites = vec![
    "https://company1.com",
    "https://company2.com",
    "https://company3.com",
];

stream::iter(websites)

    .for_each_concurrent(50, |site| async move {

        let html = fetch(site).await.unwrap();

        println!("{}", html.len());

    })

    .await;
Excel Export
use rust_xlsxwriter::*;

let mut workbook = Workbook::new();

let worksheet = workbook.add_worksheet();

worksheet.write(0,0,"Company")?;
worksheet.write(0,1,"Email")?;
worksheet.write(0,2,"Phone")?;
worksheet.write(0,3,"Hiring")?;
worksheet.write(0,4,"Lead Score")?;

workbook.save("companies.xlsx")?;
Performance

A production version could also include:

tokio for asynchronous workers
Worker pools with bounded concurrency
Retry with exponential backoff
Respect for robots.txt and site rate limits
Deduplication by normalized domain
PostgreSQL or Neo4j storage
Redis or Kafka for job queues
Structured logging and metrics
Incremental recrawls based on timestamps or content changes

For a company like AGB Technologies, I'd treat this as a lead intelligence platform rather than just a scraper. The crawler would collect public business information, detect hiring and growth signals, score companies based on configurable criteria, and continuously update a lead database that your sales team can filter and export to Excel. This approach is more scalable and maintainable than a one-off scraping script, while also being easier to operate responsibly within the constraints of the websites you crawl.

also want to automate  Seed URLs 

Yes. I would automate Seed URL discovery instead of maintaining a static list. The goal is to continuously discover new company websites from public sources, then feed them into your crawler.

A pipeline could look like this:

                     Search Queries
                           │
      ┌────────────────────┼────────────────────┐
      │                    │                    │
      ▼                    ▼                    ▼
 Search Engine      Business Directories    Startup Lists
      │                    │                    │
      └───────────────┬────┴────────────────────┘
                      ▼
              URL Normalization
                      ▼
            Duplicate Removal
                      ▼
           Company Discovery Queue
                      ▼
                Rust Crawlers
                      ▼
             Company Intelligence
Seed Generation

Instead of storing URLs, store search strategies.

Example:

country:
  - usa
  - uk

industry:
  - software development
  - web development
  - cloud consulting
  - managed services
  - AI consulting
  - ERP
  - CRM
  - DevOps

city:
  - New York
  - Boston
  - Chicago
  - Austin
  - London
  - Manchester
  - Birmingham

Generate searches like:

software development company New York

cloud consulting London

managed IT services Boston

DevOps consulting Austin

ERP implementation UK

CRM consulting Manchester

These become discovery tasks rather than hard-coded URLs.

Rust Discovery Queue
struct DiscoveryTask {
    keyword: String,
    country: String,
    city: String,
}

Generate thousands of tasks automatically.

Normalize URLs

Many sources point to the same company.

https://company.com

https://company.com/

https://www.company.com

http://company.com

Normalize to

company.com

Then hash it.

use sha2::{Digest, Sha256};

let hash = Sha256::digest(domain.as_bytes());

No duplicate crawling.

Crawl Budget

Each company receives a budget.

/

contact

about

careers

team

services

privacy

Maximum 20 pages.

Avoid infinite crawling.

Incremental Updates

Store

last_crawled

etag

last_modified

content_hash

Skip unchanged websites.

Redis Queue
DISCOVER

↓

FETCH

↓

PARSE

↓

ENRICH

↓

SCORE

↓

SAVE

↓

EXPORT

Each stage is independent.

Scoring During Discovery

Immediately reject websites like

blogspot.com

wordpress.com

facebook.com

linkedin.com

youtube.com

medium.com

Keep only

company.com

company.co.uk

company.io

company.ai
Database
Companies

CompanyPages

Contacts

HiringSignals

Technologies

LeadScores

Exports

CrawlLogs
Continuous Discovery

Run discovery every day:

00:00 Generate search tasks

00:15 Discover new companies

01:00 Crawl websites

03:00 Score leads

04:00 Export Excel

05:00 Sales CRM Sync
One important consideration

For the discovery stage, avoid scraping search engine result pages directly at scale, as that often violates their terms of service. A more robust approach is to use:

Search APIs where available.
Public business directories that permit indexing or provide APIs.
Official company registries.
Companies' own websites and career pages.

This gives you a more reliable and maintainable system while respecting the rules of the data sources you're using.