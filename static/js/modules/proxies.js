import { API_ENDPOINTS } from '../config.js';
import { renderPaginationUI } from '../components/pagination.js';

let proxiesPage = 1;
let proxiesLimit = 25;
let cachedProxies = [];
let sortCol = 'id';
let sortAsc = true;

export async function loadProxies(newPage, newLimit) {
    if (newPage) proxiesPage = newPage;
    if (newLimit) proxiesLimit = newLimit;

    try {
        const res = await fetch(API_ENDPOINTS.PROXIES);
        const data = await res.json();
        cachedProxies = data.proxies || [];
        renderProxyTable();
    } catch (err) {
        console.error("Error loading proxies:", err);
    }
}

window.sortProxies = function(col) {
    if (sortCol === col) {
        sortAsc = !sortAsc;
    } else {
        sortCol = col;
        sortAsc = true;
    }
    renderProxyTable();
};

function renderProxyTable() {
    const tbody = document.getElementById('proxyTableBody');
    if (!tbody) return;
    tbody.innerHTML = '';

    let sorted = [...cachedProxies];

    sorted.sort((a, b) => {
        let valA, valB;
        if (sortCol === 'success_rate') {
            const totA = (a.success_count || 0) + (a.fail_count || 0);
            const totB = (b.success_count || 0) + (b.fail_count || 0);
            valA = totA > 0 ? ((a.success_count / totA) * 100) : 100;
            valB = totB > 0 ? ((b.success_count / totB) * 100) : 100;
        } else {
            valA = a[sortCol];
            valB = b[sortCol];
        }

        if (typeof valA === 'string') {
            return sortAsc ? valA.localeCompare(valB) : valB.localeCompare(valA);
        } else {
            return sortAsc ? (valA - valB) : (valB - valA);
        }
    });

    const total = sorted.length;
    renderPaginationUI('proxies', total, proxiesPage, proxiesLimit, (p, l) => loadProxies(p, l));

    const startIndex = (proxiesPage - 1) * proxiesLimit;
    const pageProxies = sorted.slice(startIndex, startIndex + proxiesLimit);

    if (pageProxies.length === 0) {
        tbody.innerHTML = `<tr><td colspan="8" style="text-align:center; padding: 24px; color: var(--text-muted);">No proxies found in rotation pool.</td></tr>`;
        return;
    }

    pageProxies.forEach(p => {
        const totalReqs = (p.success_count || 0) + (p.fail_count || 0);
        const ratePct = totalReqs > 0 ? Math.round((p.success_count / totalReqs) * 100) : 100;
        const rateBadge = ratePct >= 90 ? 'badge-high' : (ratePct >= 60 ? 'badge-medium' : 'badge-low');

        const tr = document.createElement('tr');
        tr.innerHTML = `
            <td>${p.id}</td>
            <td><strong style="color:var(--text-primary);">${p.url}</strong></td>
            <td><span class="badge" style="background:#F3F4F6; color:var(--text-primary); font-weight:600;">${p.protocol}</span></td>
            <td><span class="badge ${p.active ? 'badge-high' : 'badge-low'}">${p.active ? 'Active' : 'Inactive'}</span></td>
            <td><strong style="color:#10B981;">${p.success_count}</strong></td>
            <td><span style="color:#EF4444;">${p.fail_count}</span></td>
            <td><strong>${p.latency_ms} ms</strong></td>
            <td><span class="badge ${rateBadge}">${ratePct}%</span></td>
        `;
        tbody.appendChild(tr);
    });
}
