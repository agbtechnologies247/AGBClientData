import { API_ENDPOINTS } from './js/config.js';
import { initRoutes } from './js/routes.js';
import { loadStats, loadLeads } from './js/modules/dashboard.js';
import { loadPipeline } from './js/modules/pipeline.js';
import { loadOutreachHistory } from './js/modules/outreach.js';
import { loadLogs, initLogsHandlers } from './js/modules/logs.js';
import { loadProxies } from './js/modules/proxies.js';

let peoplePage = 1, peopleLimit = 25;
let investorsPage = 1, investorsLimit = 25;

// Global Toast Helper
window.showToast = function(msg) {
    const toast = document.getElementById('copyToast');
    if (!toast) return;
    document.getElementById('toastMsg').innerText = msg;
    toast.classList.add('show');
    setTimeout(() => {
        toast.classList.remove('show');
    }, 2500);
};

// Global 1-Click Copy helper
window.copyToClipboard = function(text, label) {
    if (!text || text === '-') return;
    try {
        if (navigator.clipboard && window.isSecureContext) {
            navigator.clipboard.writeText(text).then(() => {
                showToast(`Copied ${label || 'text'}: ${text}`);
            }).catch(() => {
                fallbackCopy(text, label);
            });
        } else {
            fallbackCopy(text, label);
        }
    } catch (err) {
        fallbackCopy(text, label);
    }
};

function fallbackCopy(text, label) {
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.position = "fixed";
    textArea.style.top = "0";
    textArea.style.left = "0";
    textArea.style.opacity = "0";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    try {
        document.execCommand('copy');
        showToast(`Copied ${label || 'text'}: ${text}`);
    } catch (err) {
        console.error('Fallback copy failed', err);
    }
    document.body.removeChild(textArea);
}

document.addEventListener('DOMContentLoaded', () => {
    // Initial Load
    loadStats();
    loadLeads();

    // Initialize Routes & Tab Switcher
    initRoutes((tabId) => {
        if (tabId === 'tabDashboard') { loadStats(); loadLeads(); }
        else if (tabId === 'tabPipeline') { loadPipeline(); }
        else if (tabId === 'tabPeople') { loadPeople(); }
        else if (tabId === 'tabInvestors') { loadInvestors(); }
        else if (tabId === 'tabOutreach') { loadOutreachHistory(); }
        else if (tabId === 'tabProxies') { loadProxies(); }
        else if (tabId === 'tabLogs') { loadLogs(); }
    });

    // Initialize Log Clear Handlers
    initLogsHandlers(window.showToast);

    // Toggle Crawler (Start / Stop)
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
                if (res.ok) {
                    showToast("Continuous Crawl Daemon started in Stealth Mode.");
                }
            } else {
                await fetch(API_ENDPOINTS.CRAWLER_STOP, { method: 'POST' });
                showToast("Crawler daemon stop signal sent.");
            }
            loadStats();
        });
    }

    // Save Stealth Settings Form
    document.getElementById('formCrawlerSettings')?.addEventListener('submit', async (e) => {
        e.preventDefault();
        const mode = document.getElementById('settingMode')?.value || 'stealth';
        try {
            const res = await fetch(API_ENDPOINTS.CRAWLER_START, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ seed_urls: [], mode: mode })
            });
            if (res.ok) {
                showToast(`Stealth settings updated! Mode set to ${mode.toUpperCase()}.`);
                loadStats();
            }
        } catch (err) {
            alert("Error saving settings: " + err);
        }
    });

    // Launch Target Seeds
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
            if (res.ok) {
                showToast(`Crawl session launched with ${seeds.length > 0 ? seeds.length : 'default'} seed domains!`);
                loadStats();
            }
        } catch (err) {
            alert("Error launching crawl: " + err);
        }
    });

    // Trigger Outreach Batch Now
    document.getElementById('btnTriggerOutreachBatch')?.addEventListener('click', async () => {
        try {
            const res = await fetch(API_ENDPOINTS.OUTREACH_TRIGGER, { method: 'POST' });
            if (res.ok) {
                showToast("Hostinger SMTP outreach batch triggered!");
                setTimeout(loadOutreachHistory, 2000);
            }
        } catch (err) {
            alert("Error triggering outreach batch: " + err);
        }
    });

    // Clear Database Button
    document.getElementById('btnClearDatabaseBtn')?.addEventListener('click', async () => {
        if (!confirm("Are you sure you want to clear all existing crawled leads and restore baseline data?")) {
            return;
        }
        try {
            const res = await fetch(API_ENDPOINTS.CRAWLER_CLEAR_DB, { method: 'POST' });
            if (res.ok) {
                showToast("Database reset successfully!");
                loadStats();
                loadLeads();
            }
        } catch (err) {
            console.error("Error clearing database:", err);
        }
    });

    // Filter event listeners
    document.getElementById('filterSearch')?.addEventListener('input', () => loadLeads(1));
    document.getElementById('filterCountry')?.addEventListener('change', () => loadLeads(1));
    document.getElementById('filterPriority')?.addEventListener('change', () => loadLeads(1));
    document.getElementById('filterHiringOnly')?.addEventListener('change', () => loadLeads(1));
    document.getElementById('btnRefreshLeads')?.addEventListener('click', () => loadLeads());
    document.getElementById('filterOutreachStatus')?.addEventListener('change', () => loadOutreachHistory(1));
});

// Fetch People / Decision Makers
async function loadPeople(newPage, newLimit) {
    if (newPage) peoplePage = newPage;
    if (newLimit) peopleLimit = newLimit;

    const search = document.getElementById('filterPeopleSearch')?.value || '';
    const role = document.getElementById('filterPeopleRole')?.value || 'ALL';

    const params = new URLSearchParams({
        page: peoplePage,
        limit: peopleLimit,
        role: role,
        search_query: search
    });

    try {
        const res = await fetch(`${API_ENDPOINTS.PEOPLE}?${params}`);
        const data = await res.json();
        const tbody = document.getElementById('peopleTableBody');
        if (!tbody) return;
        tbody.innerHTML = '';

        (data.people || []).forEach(p => {
            const tr = document.createElement('tr');
            const copyEmailBtn = p.public_email ? `<button class="btn-copy" onclick="copyToClipboard('${p.public_email}', 'Email')"><i class="fa-solid fa-copy"></i> Email</button>` : '';

            tr.innerHTML = `
                <td><strong>${p.name}</strong></td>
                <td><span class="badge" style="background:#F3F4F6; color:var(--text-primary); font-weight:600;">${p.title}</span></td>
                <td><strong>${p.company_name}</strong><br><small style="color:var(--text-secondary);">${p.company_domain}</small></td>
                <td><strong style="font-size:16px; color:var(--text-primary);">${p.decision_maker_score}</strong></td>
                <td>${p.public_email ? `<span style="color:#047857;"><i class="fa-solid fa-envelope"></i> ${p.public_email}</span>` : '<span style="color:var(--text-muted);">-</span>'}</td>
                <td>${copyEmailBtn}</td>
                <td>${p.linkedin_url ? `<a href="${p.linkedin_url}" target="_blank" class="btn btn-secondary btn-sm"><i class="fa-solid fa-brands fa-linkedin"></i> Profile</a>` : '-'}</td>
            `;
            tbody.appendChild(tr);
        });
    } catch (err) {
        console.error("Error loading people:", err);
    }
}

// Fetch Investors
async function loadInvestors(newPage, newLimit) {
    if (newPage) investorsPage = newPage;
    if (newLimit) investorsLimit = newLimit;

    const search = document.getElementById('filterInvSearch')?.value || '';
    const itype = document.getElementById('filterInvType')?.value || 'ALL';

    const params = new URLSearchParams({
        page: investorsPage,
        limit: investorsLimit,
        investor_type: itype,
        search_query: search
    });

    try {
        const res = await fetch(`${API_ENDPOINTS.INVESTORS}?${params}`);
        const data = await res.json();
        const tbody = document.getElementById('investorTableBody');
        if (!tbody) return;
        tbody.innerHTML = '';

        (data.investors || []).forEach(inv => {
            const tr = document.createElement('tr');
            const focusBadges = (inv.focus || []).map(f => `<span class="badge" style="background:var(--accent-light); color:var(--accent); font-size:11px; margin-right:4px;">${f}</span>`).join('');
            const copyEmailBtn = inv.public_email ? `<button class="btn-copy" onclick="copyToClipboard('${inv.public_email}', 'Email')"><i class="fa-solid fa-copy"></i> Email</button>` : '';

            tr.innerHTML = `
                <td>
                    <a href="${inv.website}" target="_blank" style="color:var(--text-primary); text-decoration:none; font-weight:700;">
                        ${inv.name} <i class="fa-solid fa-arrow-up-right-from-square" style="font-size:10px; color:var(--accent);"></i>
                    </a>
                </td>
                <td><span class="badge badge-high">${inv.investor_type}</span><br><small style="color:var(--text-secondary);">${inv.country}</small></td>
                <td><strong style="font-size:16px; color:var(--text-primary);">${inv.score}</strong></td>
                <td><strong>${inv.check_size || 'N/A'}</strong></td>
                <td>${inv.public_email ? `<span style="color:#047857;"><i class="fa-solid fa-envelope"></i> ${inv.public_email}</span>` : '<span style="color:var(--text-muted);">-</span>'}</td>
                <td>${copyEmailBtn}</td>
                <td>${focusBadges}</td>
                <td><span class="badge badge-medium">Active Portfolio</span></td>
            `;
            tbody.appendChild(tr);
        });
    } catch (err) {
        console.error("Error loading investors:", err);
    }
}
