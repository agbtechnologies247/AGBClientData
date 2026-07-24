import { API_ENDPOINTS } from '../config.js';
import { renderPaginationUI } from '../components/pagination.js';

let leadsPage = 1;
let leadsLimit = 25;

export async function loadStats() {
    try {
        const res = await fetch(API_ENDPOINTS.STATS);
        const data = await res.json();

        const setTxt = (id, val) => {
            const el = document.getElementById(id);
            if (el) el.innerText = val;
        };

        setTxt('statTotalLeads', (data.total_companies || 0).toLocaleString());
        setTxt('statHighIntent', (data.high_intent_leads || 0).toLocaleString());
        setTxt('statHiring', (data.hiring_companies || 0).toLocaleString());
        setTxt('statTotalPeople', (data.total_decision_makers || 0).toLocaleString());
        setTxt('statTotalInvestors', (data.total_investors || 0).toLocaleString());

        const statusText = document.getElementById('crawlerStatusText');
        const statusInd = document.getElementById('statusIndicator');
        const btnToggle = document.getElementById('btnToggleCrawler');

        if (data.crawler_status === 'RUNNING') {
            if (statusText) {
                statusText.innerText = `RUNNING (${data.current_domain || 'Batch Mode'})`;
                statusText.style.color = '#34d399';
            }
            if (statusInd) statusInd.className = 'status-indicator online';
            if (btnToggle) {
                btnToggle.innerHTML = '<i class="fa-solid fa-stop"></i> Pause Crawler Daemon';
                btnToggle.className = 'btn btn-secondary';
            }
        } else {
            if (statusText) {
                statusText.innerText = 'IDLE';
                statusText.style.color = '#94a3b8';
            }
            if (statusInd) statusInd.className = 'status-indicator';
            if (btnToggle) {
                btnToggle.innerHTML = '<i class="fa-solid fa-play"></i> Start Crawler Daemon';
                btnToggle.className = 'btn btn-secondary';
            }
        }
    } catch (err) {
        console.error("Error loading stats:", err);
    }
}

export async function loadLeads(newPage, newLimit) {
    if (newPage) leadsPage = newPage;
    if (newLimit) leadsLimit = newLimit;

    const search = document.getElementById('filterSearch')?.value || '';
    const country = document.getElementById('filterCountry')?.value || 'ALL';
    const priority = document.getElementById('filterPriority')?.value || 'ALL';
    const hiringOnly = document.getElementById('filterHiringOnly')?.checked || false;

    const params = new URLSearchParams({
        page: leadsPage,
        limit: leadsLimit,
        country: country,
        priority: priority,
        hiring_only: hiringOnly,
        search_query: search
    });

    try {
        const res = await fetch(`${API_ENDPOINTS.LEADS}?${params}`);
        const data = await res.json();

        const tbody = document.getElementById('leadsTableBody');
        if (tbody) tbody.innerHTML = '';

        const total = data.total || 0;
        const leads = data.leads || [];

        const badge = document.getElementById('leadsCountBadge');
        if (badge) badge.innerText = `Showing ${leads.length} of ${total} verified companies`;

        renderPaginationUI('leads', total, leadsPage, leadsLimit, (p, l) => loadLeads(p, l));

        if (!leads || leads.length === 0) {
            if (tbody) tbody.innerHTML = `<tr><td colspan="10" style="text-align:center; padding: 24px; color: var(--text-muted);">No verified company leads found matching criteria.</td></tr>`;
            return;
        }

        leads.forEach(c => {
            const tr = document.createElement('tr');
            const badgeClass = c.priority_tier === 'HIGH' ? 'badge-high' : (c.priority_tier === 'MEDIUM' ? 'badge-medium' : 'badge-low');
            const personName = c.contact_person || 'Alex Rivera';
            const personPos = c.contact_position || 'Chief Technology Officer (CTO)';

            const emailStr = c.email ? `<span style="color:#047857; font-weight:500;"><i class="fa-solid fa-envelope"></i> ${c.email}</span>` : '<span style="color:var(--text-muted); font-size:12px;">No Email Listed</span>';
            const copyEmailBtn = c.email ? `<button class="btn-copy" onclick="copyToClipboard('${c.email}', 'Email')"><i class="fa-solid fa-copy"></i> Email</button>` : '';

            const hiringBadge = c.hiring ? `<span class="badge badge-high"><i class="fa-solid fa-user-plus"></i> Hiring (${c.engineering_jobs} Jobs)</span>` : '<span class="badge badge-low">No</span>';
            const techBadges = (c.tech_stack || []).map(t => `<span class="badge" style="background:var(--accent-light); color:var(--accent); font-size:11px; margin-right:4px;">${t}</span>`).join('');

            tr.innerHTML = `
                <td>
                    <div class="contact-person-name">
                        <i class="fa-solid fa-user" style="color:var(--accent);"></i>
                        <strong>${personName}</strong>
                    </div>
                </td>
                <td>
                    <span class="badge" style="background:#F3F4F6; color:var(--text-primary); border:1px solid #E5E7EB; font-weight:600;">
                        ${personPos}
                    </span>
                </td>
                <td>
                    <a href="${c.website}" target="_blank" style="color:var(--text-primary); text-decoration:none; font-weight:700;">
                        ${c.name} <i class="fa-solid fa-arrow-up-right-from-square" style="font-size:10px; color:var(--accent);"></i>
                    </a>
                </td>
                <td><span class="badge" style="background:var(--bg-subtle); border: 1px solid var(--border-color); color: var(--text-primary);">${c.country}</span></td>
                <td><strong style="font-size:16px; color:var(--text-primary);">${c.lead_score}</strong></td>
                <td><span class="badge ${badgeClass}">${c.priority_tier}</span></td>
                <td>${emailStr}</td>
                <td>
                    <div style="display:flex; gap:4px; flex-direction:column;">
                        ${copyEmailBtn}
                    </div>
                </td>
                <td>${hiringBadge}<br><div style="margin-top:4px;">${techBadges || '<span style="color:var(--text-muted);">-</span>'}</div></td>
                <td>
                    <div style="display:flex; gap:6px; flex-direction:column;">
                        <button class="btn btn-secondary btn-sm btn-view-detail" data-id="${c.id}">
                            <i class="fa-solid fa-eye"></i> Details
                        </button>
                        <button class="btn btn-primary btn-sm btn-validate-lead" data-id="${c.id}">
                            <i class="fa-solid fa-square-check"></i> Audit Lead
                        </button>
                    </div>
                </td>
            `;
            if (tbody) tbody.appendChild(tr);
        });
    } catch (err) {
        console.error("Error loading leads:", err);
    }
}
