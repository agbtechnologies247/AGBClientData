import { API_ENDPOINTS } from '../config.js';

export async function loadProxies() {
    try {
        const res = await fetch(API_ENDPOINTS.PROXIES);
        const data = await res.json();
        const tbody = document.getElementById('proxyTableBody');
        if (!tbody) return;
        tbody.innerHTML = '';

        (data.proxies || []).forEach(p => {
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td>${p.id}</td>
                <td><strong>${p.url}</strong></td>
                <td><span class="badge" style="background:rgba(255,255,255,0.08);">${p.protocol}</span></td>
                <td><span class="badge ${p.active ? 'badge-high' : 'badge-low'}">${p.active ? 'Active' : 'Inactive'}</span></td>
                <td>${p.success_count}</td>
                <td>${p.fail_count}</td>
                <td>${p.latency_ms} ms</td>
            `;
            tbody.appendChild(tr);
        });
    } catch (err) {
        console.error("Error loading proxies:", err);
    }
}
