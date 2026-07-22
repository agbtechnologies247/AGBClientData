# LeadPulse - US & UK Marketing Data Crawler & Lead Intelligence Platform

High-performance, anti-blocking asynchronous Rust server paired with a modern 1-page Web Dashboard for discovering, crawling, scoring, filtering, and exporting US/UK B2B IT & service company leads for sales outreach.

---

## Features

- **Automated Target Discovery**: Continuous query matrix synthesis for US and UK tech hubs with automatic domain validation and noise filtering.
- **Anti-Blocking Security Engine**:
  - Proxy Pool Manager supporting HTTP, HTTPS, and SOCKS5 round-robin rotation.
  - User-Agent & Header Rotation emulating real desktop Chrome, Firefox, Edge, and Safari browser signatures.
  - Randomized Human Behavior Delay Jitter (1.5s - 4.0s) and per-domain concurrency bounds.
- **Lead Scoring & Intent Detection**:
  - Automatically extracts emails (`info@`, `sales@`, `contact@`), phone numbers, contact pages, and LinkedIn URLs.
  - Detects active software engineering job openings and remote positions (+20 score bonus).
  - Identifies offshore & outsourcing intent keywords ("development partner", "dedicated team", "contractors") with a +80 priority score boost.
- **Lead Intelligence Dashboard**:
  - Single Page Application (SPA) served directly from the Rust binary.
  - Real-time stats dashboard, filter controls (Min Score, Country, Hiring, Has Email/Phone), and company detail view.
- **Excel & CSV Export**: Download formatted multi-tab `.xlsx` workbooks organized by High, Medium, and Low priority tiers.

---

## Deployment Options on VPS

### Option A: Using Docker & Docker Compose (Recommended)

1. **Clone repository onto VPS**:
   ```bash
   git clone <your-repo-url> /opt/marketing-crawler
   cd /opt/marketing-crawler
   ```

2. **Launch with Docker Compose**:
   ```bash
   docker-compose up -d --build
   ```

3. **Access Dashboard**:
   Open browser at `http://<YOUR_VPS_IP>:8080`

---

### Option B: Direct Systemd Deployment on Linux VPS

1. **Build Release Binary**:
   ```bash
   cargo build --release
   ```

2. **Copy Files**:
   ```bash
   mkdir -p /opt/marketing-crawler
   cp target/release/marketing-data-crawler /opt/marketing-crawler/
   cp -r static /opt/marketing-crawler/
   cp marketing-crawler.service /etc/systemd/system/
   ```

3. **Enable & Start Systemd Service**:
   ```bash
   systemctl daemon-reload
   systemctl enable marketing-crawler
   systemctl start marketing-crawler
   ```

4. **Check Logs**:
   ```bash
   journalctl -u marketing-crawler -f
   ```

---

## API Reference

- `GET /api/stats` - Get live lead intelligence stats and crawler state.
- `GET /api/leads` - List and filter leads (`country`, `priority`, `hiring_only`, `has_email`, `search_query`).
- `POST /api/crawler/start` - Launch crawl engine with custom seeds or auto-discovered seeds.
- `POST /api/crawler/stop` - Stop active crawling session.
- `POST /api/crawler/auto-seeds` - Trigger automated seed discovery loop.
- `GET /api/proxies` - List registered anti-blocking proxies.
- `POST /api/proxies` - Register new HTTP/SOCKS5 proxies to rotation pool.
- `GET /api/leads/export` - Export leads as a styled `.xlsx` Excel file.
