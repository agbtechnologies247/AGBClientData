import { API_ENDPOINTS } from '../config.js';
import { renderPaginationUI } from '../components/pagination.js';

let peoplePage = 1;
let peopleLimit = 25;

export async function loadPeople(newPage, newLimit) {
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

        const total = data.total !== undefined ? data.total : (data.people || []).length;
        const badge = document.getElementById('peopleCountBadge');
        if (badge) badge.innerText = `${total.toLocaleString()} decision makers`;

        renderPaginationUI('people', total, peoplePage, peopleLimit, (p, l) => loadPeople(p, l));

        const people = data.people || [];
        if (people.length === 0) {
            tbody.innerHTML = `<tr><td colspan="7" style="text-align:center; padding: 24px; color: var(--text-muted);">No decision makers found matching criteria.</td></tr>`;
            return;
        }

        people.forEach(p => {
            const tr = document.createElement('tr');
            const copyEmailBtn = p.public_email ? `<button class="btn-copy" onclick="copyToClipboard('${p.public_email}', 'Email')"><i class="fa-solid fa-copy"></i> Copy</button>` : '<span style="color:var(--text-muted);">-</span>';

            tr.innerHTML = `
                <td>
                    <div class="contact-person-name">
                        <i class="fa-solid fa-user-tie" style="color:var(--accent);"></i>
                        <strong>${p.name}</strong>
                    </div>
                </td>
                <td><span class="badge" style="background:#F3F4F6; color:var(--text-primary); border:1px solid #E5E7EB; font-weight:600;">${p.title}</span></td>
                <td><strong>${p.company_name}</strong><br><small style="color:var(--text-secondary);">${p.company_domain}</small></td>
                <td><strong style="font-size:16px; color:var(--text-primary);">${p.decision_maker_score}</strong></td>
                <td>${p.public_email ? `<span style="color:#047857; font-weight:500;"><i class="fa-solid fa-envelope"></i> ${p.public_email}</span>` : '<span style="color:var(--text-muted); font-size:12px;">No Email</span>'}</td>
                <td>${copyEmailBtn}</td>
                <td>${p.linkedin_url ? `<a href="${p.linkedin_url}" target="_blank" class="btn btn-secondary btn-sm"><i class="fa-solid fa-brands fa-linkedin"></i> Profile</a>` : '-'}</td>
            `;
            tbody.appendChild(tr);
        });
    } catch (err) {
        console.error("Error loading decision makers:", err);
    }
}
