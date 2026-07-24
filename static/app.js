import { API_ENDPOINTS } from './js/config.js';
import { initRoutes } from './js/routes.js';
import { loadStats, loadLeads } from './js/modules/dashboard.js';
import { loadPipeline } from './js/modules/pipeline.js';
import { loadPeople } from './js/modules/people.js';
import { loadInvestors, initInvestorHandlers } from './js/modules/investors.js';
import { loadOutreachHistory } from './js/modules/outreach.js';
import { initCrawlerHandlers } from './js/modules/crawler.js';
import { loadProxies } from './js/modules/proxies.js';
import { loadLogs, initLogsHandlers } from './js/modules/logs.js';

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

    // Auto-refresh stats & crawler status badge every 5 seconds
    setInterval(loadStats, 5000);

    // Initialize Routes & Tab Switcher for ALL 8 modules
    initRoutes((tabId) => {
        const t = tabId.startsWith('tab') ? tabId : `tab${tabId.charAt(0).toUpperCase() + tabId.slice(1)}`;
        if (t === 'tabDashboard') { loadStats(); loadLeads(); }
        else if (t === 'tabPipeline') { loadPipeline(); }
        else if (t === 'tabPeople') { loadPeople(); }
        else if (t === 'tabInvestors') { loadInvestors(); }
        else if (t === 'tabCampaigns' || t === 'tabOutreach') { loadOutreachHistory(); }
        else if (t === 'tabProxies') { loadProxies(); }
        else if (t === 'tabLogs') { loadLogs(); }
    });

    // Initialize Module Handlers
    initLogsHandlers(window.showToast);
    initInvestorHandlers(window.showToast);
    initCrawlerHandlers(loadStats, window.showToast);

    // Trigger Outreach Batch Now
    document.getElementById('btnTriggerOutreachBatch')?.addEventListener('click', async () => {
        const category = document.getElementById('campaignTargetCategory')?.value || 'PEOPLE';
        try {
            const res = await fetch(API_ENDPOINTS.OUTREACH_TRIGGER, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ target_category: category, limit: 1000 })
            });
            if (res.ok) {
                showToast(`Hostinger SMTP outreach batch of 1,000 emails triggered for ${category}!`);
                setTimeout(loadOutreachHistory, 2000);
            }
        } catch (err) {
            alert("Error triggering outreach batch: " + err);
        }
    });

    // Modal Close Handlers
    document.getElementById('btnCloseModal')?.addEventListener('click', () => {
        document.getElementById('leadModal')?.classList.remove('active');
    });

    document.getElementById('btnCloseValidateModal')?.addEventListener('click', () => {
        document.getElementById('validateLeadModal')?.classList.remove('active');
    });

    document.getElementById('btnCloseHtmlModal')?.addEventListener('click', () => {
        document.getElementById('htmlPreviewModal')?.classList.remove('active');
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

    // Export Buttons
    document.getElementById('btnExportExcel')?.addEventListener('click', () => { window.location.href = API_ENDPOINTS.EXPORT_LEADS; });
    document.getElementById('btnExportPeople')?.addEventListener('click', () => { window.location.href = API_ENDPOINTS.EXPORT_PEOPLE; });
    document.getElementById('btnExportInvestors')?.addEventListener('click', () => { window.location.href = API_ENDPOINTS.EXPORT_INVESTORS; });
    document.getElementById('btnRefreshPipeline')?.addEventListener('click', () => loadPipeline());

    // Filter event listeners across modules
    document.getElementById('filterSearch')?.addEventListener('input', () => loadLeads(1));
    document.getElementById('filterCountry')?.addEventListener('change', () => loadLeads(1));
    document.getElementById('filterPriority')?.addEventListener('change', () => loadLeads(1));
    document.getElementById('filterHiringOnly')?.addEventListener('change', () => loadLeads(1));
    document.getElementById('btnRefreshLeads')?.addEventListener('click', () => loadLeads());
    document.getElementById('filterOutreachStatus')?.addEventListener('change', () => loadOutreachHistory(1));
    document.getElementById('filterPeopleSearch')?.addEventListener('input', () => loadPeople(1));
    document.getElementById('filterPeopleRole')?.addEventListener('change', () => loadPeople(1));
    document.getElementById('filterInvSearch')?.addEventListener('input', () => loadInvestors(1));
    document.getElementById('filterInvType')?.addEventListener('change', () => loadInvestors(1));
    document.getElementById('filterInvFocus')?.addEventListener('change', () => loadInvestors(1));
});
