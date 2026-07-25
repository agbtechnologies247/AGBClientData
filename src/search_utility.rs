use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use urlencoding::encode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub results: Vec<SearchResult>,
}

pub struct LiveSearchEngine {
    client: Client,
}

impl LiveSearchEngine {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(true)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    /// Phase 3: Fetch web search data asynchronously via API or live fallback
    pub async fn fetch_web_data(&self, query: &str, api_key: Option<&str>) -> Result<ApiResponse, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(key) = api_key {
            if !key.trim().is_empty() {
                let url = format!("https://google.serper.dev/search");
                let payload = serde_json::json!({
                    "q": query,
                    "num": 20
                });

                let resp = self.client
                    .post(&url)
                    .header("X-API-KEY", key)
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;

                if resp.status().is_success() {
                    let body: serde_json::Value = resp.json().await?;
                    let mut results = Vec::new();

                    if let Some(organic) = body.get("organic").and_then(|v| v.as_array()) {
                        for item in organic {
                            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let link = item.get("link").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let snippet = item.get("snippet").and_then(|v| v.as_str()).unwrap_or("").to_string();

                            if !link.is_empty() {
                                results.push(SearchResult { title, link, snippet });
                            }
                        }
                    }

                    return Ok(ApiResponse { results });
                }
            }
        }

        // Live fallback HTML search (DuckDuckGo Lite / HTML)
        self.fetch_duckduckgo_fallback(query).await
    }

    /// Fallback live web search crawler
    async fn fetch_duckduckgo_fallback(&self, query: &str) -> Result<ApiResponse, Box<dyn std::error::Error + Send + Sync>> {
        let encoded_q = encode(query);
        let target_url = format!("https://html.duckduckgo.com/html/?q={}", encoded_q);

        let resp = self.client
            .get(&target_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .send()
            .await?;

        let html_body = resp.text().await?;
        let document = Html::parse_document(&html_body);

        let result_selector = Selector::parse(".result__body").unwrap();
        let title_selector = Selector::parse(".result__title a").unwrap();
        let snippet_selector = Selector::parse(".result__snippet").unwrap();

        let mut results = Vec::new();

        for element in document.select(&result_selector) {
            let title = element.select(&title_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let link = element.select(&title_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|href| {
                    if href.starts_with("//duckduckgo.com/l/?uddg=") {
                        if let Some(pos) = href.find("uddg=") {
                            let raw = &href[pos + 5..];
                            let end = raw.find('&').unwrap_or(raw.len());
                            urlencoding::decode(&raw[..end]).unwrap_or_default().to_string()
                        } else {
                            href.to_string()
                        }
                    } else {
                        href.to_string()
                    }
                })
                .unwrap_or_default();

            let snippet = element.select(&snippet_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if !link.is_empty() && !title.is_empty() {
                results.push(SearchResult { title, link, snippet });
            }
        }

        Ok(ApiResponse { results })
    }

    /// Phase 4: Article Content Extractor using Scraper
    pub fn parse_article_content(&self, html_content: &str) -> Vec<String> {
        let document = Html::parse_document(html_content);
        if let Ok(selector) = Selector::parse("p") {
            document
                .select(&selector)
                .map(|element| element.text().collect::<String>().trim().to_string())
                .filter(|text| !text.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Standalone helper matching Phase 3 signature
pub async fn fetch_web_data(query: &str, api_key: &str) -> Result<ApiResponse, reqwest::Error> {
    let client = Client::new();
    let url = format!("https://api.example.com/search?q={}", query);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;

    Ok(response)
}

/// Standalone helper matching Phase 4 signature
pub async fn parse_article_content(html_content: &str) -> Vec<String> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("p").unwrap();

    document
        .select(&selector)
        .map(|element| element.inner_html())
        .collect()
}
