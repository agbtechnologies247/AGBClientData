import { API_ENDPOINTS } from '../config.js';

export function initCrawlerHandlers(loadStats, showToast) {
    const btnToggleCrawler = document.getElementById('btnToggleCrawler');
    if (btnToggleCrawler) {
        btnToggleCrawler.addEventListener('click', async () => {
            const statusEl = document.getElementById('crawlerStatusText');
            const statusText = statusEl ? statusEl.innerText : 'IDLE';

            if (statusText.includes('IDLE')) {
                const res = await fetch(API_ENDPOINTS.CRAWLER_START, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ seed_urls: [], mode: 'stealth' })
                });
                if (res.ok && showToast) {
                    showToast("Continuous Crawl Daemon started in Stealth Mode.");
                }
            } else {
                await fetch(API_ENDPOINTS.CRAWLER_STOP, { method: 'POST' });
                if (showToast) showToast("Crawler daemon stop signal sent.");
            }
            if (loadStats) loadStats();
        });
    }

    document.getElementById('formCrawlerSettings')?.addEventListener('submit', async (e) => {
        e.preventDefault();
        const mode = document.getElementById('settingMode')?.value || 'stealth';
        try {
            const res = await fetch(API_ENDPOINTS.CRAWLER_START, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ seed_urls: [], mode: mode })
            });
            if (res.ok && showToast) {
                showToast(`Stealth settings updated! Mode set to ${mode.toUpperCase()}.`);
                if (loadStats) loadStats();
            }
        } catch (err) {
            alert("Error saving settings: " + err);
        }
    });

    document.getElementById('btnLaunchCrawl')?.addEventListener('click', async () => {
        const seedText = document.getElementById('seedUrlsArea')?.value || '';
        const seeds = seedText.split('\n').map(s => s.trim()).filter(s => s.length > 0);
        const mode = document.getElementById('settingMode')?.value || 'stealth';

        try {
            const res = await fetch(API_ENDPOINTS.CRAWLER_START, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ seed_urls: seeds, mode: mode })
            });
            if (res.ok && showToast) {
                showToast(`Crawl session launched with ${seeds.length > 0 ? seeds.length : 'default'} seed domains!`);
                if (loadStats) loadStats();
            }
        } catch (err) {
            alert("Error launching crawl: " + err);
        }
    });
}
