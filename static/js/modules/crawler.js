import { API_ENDPOINTS } from '../config.js';

export function initCrawlerHandlers(loadStats, showToast) {
    const btnToggleCrawler = document.getElementById('btnToggleCrawler');
    if (btnToggleCrawler) {
        btnToggleCrawler.addEventListener('click', async () => {
            const statusEl = document.getElementById('crawlerStatusText');
            const isIdle = statusEl ? statusEl.innerText.includes('IDLE') : true;

            btnToggleCrawler.disabled = true;

            if (isIdle) {
                btnToggleCrawler.innerHTML = '<i class="fa-solid fa-spinner fa-spin"></i> Starting Crawl Daemon...';
                if (statusEl) {
                    statusEl.innerText = 'RUNNING (Initializing...)';
                    statusEl.style.color = '#34d399';
                }
                try {
                    const res = await fetch(API_ENDPOINTS.CRAWLER_START, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ seed_urls: [], mode: 'stealth' })
                    });
                    if (res.ok && showToast) {
                        showToast("Continuous Crawl Daemon started in Stealth Mode.");
                    }
                } catch (err) {
                    console.error("Error starting crawler daemon:", err);
                }
            } else {
                btnToggleCrawler.innerHTML = '<i class="fa-solid fa-spinner fa-spin"></i> Stopping Crawler...';
                try {
                    await fetch(API_ENDPOINTS.CRAWLER_STOP, { method: 'POST' });
                    if (showToast) showToast("Crawler daemon stop signal sent.");
                } catch (err) {
                    console.error("Error stopping crawler daemon:", err);
                }
            }

            btnToggleCrawler.disabled = false;
            if (loadStats) await loadStats();
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
                if (loadStats) await loadStats();
            }
        } catch (err) {
            alert("Error saving settings: " + err);
        }
    });

    document.getElementById('btnLaunchCrawl')?.addEventListener('click', async () => {
        const btn = document.getElementById('btnLaunchCrawl');
        const seedText = document.getElementById('seedUrlsArea')?.value || '';
        const seeds = seedText.split('\n').map(s => s.trim()).filter(s => s.length > 0);
        const mode = document.getElementById('settingMode')?.value || 'stealth';

        if (btn) {
            btn.disabled = true;
            btn.innerHTML = '<i class="fa-solid fa-spinner fa-spin"></i> Launching Crawl Engine...';
        }

        try {
            const res = await fetch(API_ENDPOINTS.CRAWLER_START, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ seed_urls: seeds, mode: mode })
            });
            if (res.ok && showToast) {
                showToast(`Crawl session launched with ${seeds.length > 0 ? seeds.length + ' custom' : 'default'} seed domains!`);
                if (loadStats) await loadStats();
            }
        } catch (err) {
            alert("Error launching crawl: " + err);
        } finally {
            if (btn) {
                btn.disabled = false;
                btn.innerHTML = '<i class="fa-solid fa-play"></i> Launch Crawl Engine';
            }
        }
    });
}
