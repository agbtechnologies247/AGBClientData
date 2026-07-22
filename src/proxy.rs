use rand::seq::SliceRandom;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.5; rv:127.0) Gecko/20100101 Firefox/127.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0",
];

#[derive(Clone)]
pub struct ProxyManager {
    proxies: Arc<tokio::sync::RwLock<Vec<String>>>,
    index: Arc<AtomicUsize>,
}

impl ProxyManager {
    pub fn new(initial_proxies: Vec<String>) -> Self {
        Self {
            proxies: Arc::new(tokio::sync::RwLock::new(initial_proxies)),
            index: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn get_next_proxy(&self) -> Option<String> {
        let list = self.proxies.read().await;
        if list.is_empty() {
            return None;
        }
        let idx = self.index.fetch_add(1, Ordering::SeqCst) % list.len();
        Some(list[idx].clone())
    }

    pub async fn add_proxies(&self, new_proxies: Vec<String>) {
        let mut list = self.proxies.write().await;
        for p in new_proxies {
            if !list.contains(&p) {
                list.push(p);
            }
        }
    }

    pub async fn active_count(&self) -> usize {
        self.proxies.read().await.len()
    }

    pub fn random_user_agent(&self) -> &'static str {
        let mut rng = rand::thread_rng();
        USER_AGENTS.choose(&mut rng).unwrap_or(&USER_AGENTS[0])
    }

    pub fn build_stealth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let ua = self.random_user_agent();
        
        headers.insert(USER_AGENT, HeaderValue::from_str(ua).unwrap());
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.9,en-GB;q=0.8"),
        );
        headers.insert("Sec-Ch-Ua", HeaderValue::from_static("\"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"Google Chrome\";v=\"126\""));
        headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::from_static("?0"));
        headers.insert("Sec-Ch-Ua-Platform", HeaderValue::from_static("\"Windows\""));
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("none"));
        headers.insert("Sec-Fetch-User", HeaderValue::from_static("?1"));
        headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));

        headers
    }
}
