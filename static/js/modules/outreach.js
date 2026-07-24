import { API_ENDPOINTS } from '../config.js';
import { renderPaginationUI } from '../components/pagination.js';
import { TimeUtils } from '../utils.js';

let outreachPage = 1;
let outreachLimit = 25;

export async function loadOutreachHistory(newPage, newLimit) {
    if (newPage) outreachPage = newPage;
    if (newLimit) outreachLimit = newLimit;

    const statusFilter = document.getElementById('filterOutreachStatus')?.value || 'ALL';

    const params = new URLSearchParams({
        status: statusFilter,
        page: outreachPage,
        limit: outreachLimit
    });

    try {
        const res = await fetch(`${API_ENDPOINTS.OUTREACH_HISTORY}?${params}`);
        const data = await res.json();
        const tbody = document.getElementById('outreachHistoryTableBody');
        if (!tbody) return;
        tbody.innerHTML = '';

        const allItems = data.outreach_history || [];
        const total = data.total !== undefined ? data.total : allItems.length;

        const badge = document.getElementById('outreachCountBadge');
        if (badge) badge.innerText = `${total} ${statusFilter.toLowerCase()} emails`;

        renderPaginationUI('outreach', total, outreachPage, outreachLimit, (p, l) => loadOutreachHistory(p, l));

        if (!allItems || allItems.length === 0) {
            tbody.innerHTML = `<tr><td colspan="4" style="text-align:center; padding:20px; color:var(--text-muted);">No ${statusFilter.toLowerCase()} outreach emails found. Daemon active.</td></tr>`;
            return;
        }

        allItems.forEach(item => {
            const tr = document.createElement('tr');
            const relTime = TimeUtils.formatRelativeTime(item.sent_at);
            const fullTime = item.sent_at.slice(0, 19).replace('T', ' ');

            let badgeHtml = '<span class="badge badge-high">' + item.status + '</span>';
            if (item.status === 'DELIVERED') {
                badgeHtml = '<span class="badge" style="background:#10B981; color:#FFF; font-weight:600;"><i class="fa-solid fa-circle-check"></i> DELIVERED</span>';
            } else if (item.status === 'SENT') {
                badgeHtml = '<span class="badge" style="background:#3B82F6; color:#FFF; font-weight:600;"><i class="fa-solid fa-paper-plane"></i> SENT</span>';
            } else if (item.status === 'BOUNCED') {
                badgeHtml = '<span class="badge" style="background:#EF4444; color:#FFF; font-weight:600;"><i class="fa-solid fa-triangle-exclamation"></i> BOUNCED</span>';
            } else if (item.status === 'FAILED') {
                badgeHtml = '<span class="badge" style="background:#DC2626; color:#FFF; font-weight:600;"><i class="fa-solid fa-xmark"></i> FAILED</span>';
            } else if (item.status === 'INVALID') {
                badgeHtml = '<span class="badge" style="background:#6B7280; color:#FFF; font-weight:600;">INVALID</span>';
            } else if (item.status === 'REPLIED') {
                badgeHtml = '<span class="badge" style="background:#8B5CF6; color:#FFF; font-weight:600;"><i class="fa-solid fa-reply"></i> REPLIED</span>';
            }

            tr.innerHTML = `
                <td><strong>${item.recipient_email}</strong></td>
                <td>${item.company_name}</td>
                <td>${badgeHtml}</td>
                <td>
                    <span style="font-weight:600; color:var(--text-primary);">${relTime}</span>
                    <br><small style="color:var(--text-muted); font-size:11px;">${fullTime}</small>
                </td>
            `;
            tbody.appendChild(tr);
        });
    } catch (err) {
        console.error("Error loading outreach history:", err);
    }
}
