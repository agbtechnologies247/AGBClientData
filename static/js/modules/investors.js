import { API_ENDPOINTS } from '../config.js';
import { renderPaginationUI } from '../components/pagination.js';

let investorsPage = 1;
let investorsLimit = 25;

export async function loadInvestors(newPage, newLimit) {
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

        const total = data.total !== undefined ? data.total : (data.investors || []).length;
        const badge = document.getElementById('investorCountBadge');
        if (badge) badge.innerText = `${total.toLocaleString()} investors`;

        renderPaginationUI('investors', total, investorsPage, investorsLimit, (p, l) => loadInvestors(p, l));

        const investors = data.investors || [];
        if (investors.length === 0) {
            tbody.innerHTML = `<tr><td colspan="8" style="text-align:center; padding: 24px; color: var(--text-muted);">No verified investors found matching criteria.</td></tr>`;
            return;
        }

        investors.forEach(inv => {
            const tr = document.createElement('tr');
            const focusBadges = (inv.focus || []).map(f => `<span class="badge" style="background:var(--accent-light); color:var(--accent); font-size:11px; margin-right:4px;">${f}</span>`).join('');
            const copyEmailBtn = inv.public_email ? `<button class="btn-copy" onclick="copyToClipboard('${inv.public_email}', 'Email')"><i class="fa-solid fa-copy"></i> Copy</button>` : '<span style="color:var(--text-muted);">-</span>';

            tr.innerHTML = `
                <td>
                    <a href="${inv.website}" target="_blank" style="color:var(--text-primary); text-decoration:none; font-weight:700;">
                        ${inv.name} <i class="fa-solid fa-arrow-up-right-from-square" style="font-size:10px; color:var(--accent);"></i>
                    </a>
                </td>
                <td><span class="badge badge-high">${inv.investor_type}</span><br><small style="color:var(--text-secondary);">${inv.country}</small></td>
                <td><strong style="font-size:16px; color:var(--text-primary);">${inv.score}</strong></td>
                <td><strong>${inv.check_size || 'N/A'}</strong></td>
                <td>${inv.public_email ? `<span style="color:#047857; font-weight:500;"><i class="fa-solid fa-envelope"></i> ${inv.public_email}</span>` : '<span style="color:var(--text-muted); font-size:12px;">-</span>'}</td>
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

export function initInvestorHandlers(showToast) {
    document.getElementById('btnRunAGBMatch')?.addEventListener('click', async () => {
        try {
            const res = await fetch(API_ENDPOINTS.INVESTORS_MATCH, { method: 'POST' });
            const data = await res.json();
            if (res.ok && showToast) {
                showToast(`AI Matching completed! Ranked ${data.matches_count || 5} top investors for AGB Tech.`);
                loadInvestors();
            }
        } catch (err) {
            console.error("Error running investor match:", err);
        }
    });
}
