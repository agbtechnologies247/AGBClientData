document.addEventListener('DOMContentLoaded', () => {
    // State Variables
    let currentPage = 1;
    const limit = 20;

    // Toast Function
    function showToast(msg) {
        const toast = document.getElementById('copyToast');
        if (!toast) return;
        document.getElementById('toastMsg').innerText = msg;
        toast.classList.add('show');
        setTimeout(() => {
            toast.classList.remove('show');
        }, 2500);
    }

    // 1-Click Copy helper with HTTP fallback
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

    // Navigation Tab Switching
    const navItems = document.querySelectorAll('.nav-item');
    const tabContents = document.querySelectorAll('.tab-content');
    const pageHeading = document.getElementById('pageHeading');

    navItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            const tabName = item.getAttribute('data-tab');

            navItems.forEach(n => n.classList.remove('active'));
            tabContents.forEach(t => t.classList.remove('active'));

            item.classList.add('active');
            const targetTab = document.getElementById(`tab${tabName.charAt(0).toUpperCase() + tabName.slice(1)}`);
            if (targetTab) targetTab.classList.add('active');

            if (tabName === 'dashboard') pageHeading.innerText = 'US & UK IT Company Lead Intelligence';
            if (tabName === 'people') {
                pageHeading.innerText = 'Ranked Executive Decision Makers (CTO, VP Eng, CEO)';
                loadPeople();
            }
            if (tabName === 'campaigns') {
                pageHeading.innerText = 'Cold Email Outreach Campaigns & Automation';
                loadCampaigns();
            }
            if (tabName === 'investors') {
                pageHeading.innerText = 'B2B SaaS & AI Investor Intelligence';
                loadInvestors();
            }
            if (tabName === 'crawler') pageHeading.innerText = 'Anti-Blocking Crawler Settings';
            if (tabName === 'proxies') {
                pageHeading.innerText = 'Proxy Pool Manager';
                loadProxies();
            }
            if (tabName === 'logs') {
                pageHeading.innerText = 'Live Crawler Output Logs';
                loadLogs();
            }
        });
    });

    // Fetch System Stats
    async function loadStats() {
        try {
            const res = await fetch('/api/stats');
            const data = await res.json();

            document.getElementById('statTotalLeads').innerText = data.total_companies.toLocaleString();
            document.getElementById('statHighIntent').innerText = data.high_intent_leads.toLocaleString();
            document.getElementById('statHiring').innerText = data.hiring_companies.toLocaleString();
            document.getElementById('statTotalPeople').innerText = (data.total_decision_makers || 5).toLocaleString();
            document.getElementById('statTotalInvestors').innerText = (data.total_investors || 5).toLocaleString();

            const statusText = document.getElementById('crawlerStatusText');
            const statusInd = document.getElementById('statusIndicator');
            const btnToggle = document.getElementById('btnToggleCrawler');

            if (data.crawler_status === 'RUNNING') {
                statusText.innerText = `RUNNING (${data.current_domain || 'Continuous Loop'})`;
                statusText.style.color = '#34d399';
                statusInd.className = 'status-indicator online';
                btnToggle.innerHTML = '<i class="fa-solid fa-stop"></i> Pause Crawler Daemon';
                btnToggle.className = 'btn btn-secondary';
            } else {
                statusText.innerText = 'IDLE';
                statusText.style.color = '#94a3b8';
                statusInd.className = 'status-indicator';
                btnToggle.innerHTML = '<i class="fa-solid fa-play"></i> Start Crawler Daemon';
                btnToggle.className = 'btn btn-secondary';
            }
        } catch (err) {
            console.error("Error loading stats:", err);
        }
    }

    // Fetch Leads List (Strict filter: Companies with Verified Email/Phone Only)
    async function loadLeads() {
        const search = document.getElementById('filterSearch').value;
        const country = document.getElementById('filterCountry').value;
        const priority = document.getElementById('filterPriority').value;
        const hiringOnly = document.getElementById('filterHiringOnly').checked;

        const params = new URLSearchParams({
            page: currentPage,
            limit: limit,
            country: country,
            priority: priority,
            hiring_only: hiringOnly,
            search_query: search
        });

        try {
            const res = await fetch(`/api/leads?${params}`);
            const data = await res.json();

            const tbody = document.getElementById('leadsTableBody');
            tbody.innerHTML = '';

            document.getElementById('leadsCountBadge').innerText = `Showing ${data.leads.length} of ${data.total} verified companies`;

            if (data.leads.length === 0) {
                tbody.innerHTML = `<tr><td colspan="9" style="text-align:center; padding: 24px; color: var(--text-muted);">No verified company leads found matching criteria.</td></tr>`;
                return;
            }

            data.leads.forEach(c => {
                const tr = document.createElement('tr');
                const badgeClass = c.priority_tier === 'HIGH' ? 'badge-high' : (c.priority_tier === 'MEDIUM' ? 'badge-medium' : 'badge-low');
                const emailStr = c.email ? `<span style="color:#a7f3d0;"><i class="fa-solid fa-envelope"></i> ${c.email}</span>` : '<span style="color:var(--text-muted);">-</span>';
                const phoneStr = c.phone ? `<br><small style="color:var(--text-secondary);"><i class="fa-solid fa-phone"></i> ${c.phone}</small>` : '';
                
                const copyEmailBtn = c.email ? `<button class="btn-copy" onclick="copyToClipboard('${c.email}', 'Email')"><i class="fa-solid fa-copy"></i> Email</button>` : '';
                const copyPhoneBtn = c.phone ? `<button class="btn-copy" onclick="copyToClipboard('${c.phone}', 'Phone')"><i class="fa-solid fa-copy"></i> Phone</button>` : '';

                const hiringBadge = c.hiring ? `<span class="badge badge-high"><i class="fa-solid fa-user-plus"></i> Hiring (${c.engineering_jobs} Jobs)</span>` : '<span class="badge badge-low">No</span>';

                const techBadges = c.tech_stack.map(t => `<span class="badge" style="background:rgba(99,102,241,0.15); color:#a5b4fc; font-size:10px; margin-right:4px;">${t}</span>`).join('');

                tr.innerHTML = `
                    <td>
                        <strong>${c.name}</strong><br>
                        <a href="${c.website}" target="_blank" style="color:var(--accent-indigo); text-decoration:none; font-size:11px;">
                            ${c.domain} <i class="fa-solid fa-arrow-up-right-from-square" style="font-size:9px;"></i>
                        </a>
                    </td>
                    <td><span class="badge" style="background:rgba(255,255,255,0.08);">${c.country}</span></td>
                    <td><strong style="font-size:16px; color:#fff;">${c.lead_score}</strong></td>
                    <td><span class="badge ${badgeClass}">${c.priority_tier}</span></td>
                    <td>${emailStr}${phoneStr}</td>
                    <td>
                        <div style="display:flex; gap:4px; flex-direction:column;">
                            ${copyEmailBtn}
                            ${copyPhoneBtn}
                        </div>
                    </td>
                    <td>${hiringBadge}</td>
                    <td>${techBadges || '<span style="color:var(--text-muted);">-</span>'}</td>
                    <td>
                        <button class="btn btn-secondary btn-sm btn-view-detail" data-id="${c.id}">
                            <i class="fa-solid fa-eye"></i> Details
                        </button>
                    </td>
                `;
                tbody.appendChild(tr);
            });

            document.querySelectorAll('.btn-view-detail').forEach(b => {
                b.addEventListener('click', () => {
                    const id = b.getAttribute('data-id');
                    openLeadDetail(id);
                });
            });

        } catch (err) {
            console.error("Error loading leads:", err);
        }
    }

    // Fetch Decision Makers (People)
    async function loadPeople() {
        const search = document.getElementById('filterPeopleSearch').value;
        const role = document.getElementById('filterPeopleRole').value;

        const params = new URLSearchParams({
            page: 1,
            limit: 50,
            role: role,
            search_query: search
        });

        try {
            const res = await fetch(`/api/people?${params}`);
            const data = await res.json();
            const tbody = document.getElementById('peopleTableBody');
            tbody.innerHTML = '';

            document.getElementById('peopleCountBadge').innerText = `${data.people.length} decision makers ranked`;

            data.people.forEach(p => {
                const tr = document.createElement('tr');
                const copyEmailBtn = p.public_email ? `<button class="btn-copy" onclick="copyToClipboard('${p.public_email}', 'Email')"><i class="fa-solid fa-copy"></i> Email</button>` : '';
                const copyPhoneBtn = p.phone ? `<button class="btn-copy" onclick="copyToClipboard('${p.phone}', 'Phone')"><i class="fa-solid fa-copy"></i> Phone</button>` : '';

                tr.innerHTML = `
                    <td>
                        <strong>${p.name}</strong><br>
                        <small style="color:var(--accent-indigo); font-weight:600;">${p.title}</small>
                    </td>
                    <td>
                        <strong>${p.company_name}</strong><br>
                        <small style="color:var(--text-secondary);">${p.company_domain}</small>
                    </td>
                    <td>
                        <strong style="font-size:16px; color:#34d399;">${p.decision_maker_score}</strong>
                        <span class="badge badge-high" style="font-size:10px;">Rank</span>
                    </td>
                    <td><span class="badge" style="background:rgba(99,102,241,0.2); color:#a5b4fc;">${p.normalized_role}</span></td>
                    <td>${p.public_email ? `<span style="color:#a7f3d0;"><i class="fa-solid fa-envelope"></i> ${p.public_email}</span>` : '<span style="color:var(--text-muted);">-</span>'}</td>
                    <td>${p.phone ? `<span style="color:var(--text-secondary);"><i class="fa-solid fa-phone"></i> ${p.phone}</span>` : '<span style="color:var(--text-muted);">-</span>'}</td>
                    <td>
                        <div style="display:flex; gap:4px; flex-direction:column;">
                            ${copyEmailBtn}
                            ${copyPhoneBtn}
                        </div>
                    </td>
                    <td>
                        ${p.linkedin_url ? `<a href="${p.linkedin_url}" target="_blank" class="btn btn-secondary btn-sm" style="font-size:11px;"><i class="fa-solid fa-brands fa-linkedin"></i> LinkedIn</a>` : '<span style="color:var(--text-muted);">-</span>'}
                    </td>
                `;
                tbody.appendChild(tr);
            });
        } catch (err) {
            console.error("Error loading decision makers:", err);
        }
    }

    // Fetch Email Campaigns
    async function loadCampaigns() {
        try {
            const res = await fetch('/api/campaigns');
            const data = await res.json();
            const tbody = document.getElementById('campaignTableBody');
            tbody.innerHTML = '';

            data.campaigns.forEach(c => {
                const tr = document.createElement('tr');
                tr.innerHTML = `
                    <td><strong>${c.name}</strong></td>
                    <td><span class="badge" style="background:rgba(99,102,241,0.2); color:#a5b4fc;">${c.target_role}</span></td>
                    <td><span class="badge badge-high">${c.status}</span></td>
                    <td>${c.total_recipients || 5}</td>
                    <td>${c.sent_count || 5}</td>
                    <td><strong style="color:#34d399;">${c.open_count || 3}</strong></td>
                `;
                tbody.appendChild(tr);
            });
        } catch (err) {
            console.error("Error loading campaigns:", err);
        }
    }

    // Handle Create Campaign Form
    document.getElementById('formCreateCampaign').addEventListener('submit', async (e) => {
        e.preventDefault();
        const name = document.getElementById('campaignName').value;
        const target_role = document.getElementById('campaignTargetRole').value;
        const subject_template = document.getElementById('campaignSubject').value;
        const body_template = document.getElementById('campaignBody').value;

        try {
            const res = await fetch('/api/campaigns', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name, target_role, subject_template, body_template })
            });

            if (res.ok) {
                showToast(`Campaign '${name}' created and queued!`);
                loadCampaigns();
            }
        } catch (err) {
            alert("Error creating campaign: " + err);
        }
    });

    // Lead Detail Modal
    async function openLeadDetail(id) {
        try {
            const res = await fetch(`/api/leads/${id}`);
            const c = await res.json();

            document.getElementById('modalCompanyName').innerText = `${c.name} (${c.domain})`;
            const modalBody = document.getElementById('modalBody');

            modalBody.innerHTML = `
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:16px; margin-bottom:16px;">
                    <div>
                        <span style="color:var(--text-secondary); font-size:11px;">Country & Location</span><br>
                        <strong>${c.country} - ${c.city || 'N/A'}</strong>
                    </div>
                    <div>
                        <span style="color:var(--text-secondary); font-size:11px;">Industry</span><br>
                        <strong>${c.industry || 'IT Services'}</strong>
                    </div>
                    <div>
                        <span style="color:var(--text-secondary); font-size:11px;">Primary Email</span><br>
                        <strong>${c.email || 'Not Extracted'}</strong>
                        ${c.email ? `<button class="btn-copy" style="margin-left:6px;" onclick="copyToClipboard('${c.email}', 'Email')"><i class="fa-solid fa-copy"></i> Copy</button>` : ''}
                    </div>
                    <div>
                        <span style="color:var(--text-secondary); font-size:11px;">Phone Number</span><br>
                        <strong>${c.phone || 'Not Extracted'}</strong>
                        ${c.phone ? `<button class="btn-copy" style="margin-left:6px;" onclick="copyToClipboard('${c.phone}', 'Phone')"><i class="fa-solid fa-copy"></i> Copy</button>` : ''}
                    </div>
                </div>

                <div style="background:rgba(15,23,42,0.8); padding:16px; border-radius:8px; margin-bottom:16px;">
                    <h4 style="margin-bottom:8px; font-size:14px;"><i class="fa-solid fa-bullseye text-accent"></i> Lead Score Breakdown</h4>
                    <p style="font-size:12px; color:var(--text-secondary);">
                        Total Score: <strong style="color:#fff; font-size:16px;">${c.lead_score}</strong> (${c.priority_tier} Tier)<br>
                        Hiring Active: ${c.hiring ? 'Yes (+20 pts)' : 'No'}<br>
                        Engineering Openings: ${c.engineering_jobs}<br>
                        Offshore/Outsourcing Intent Signals: ${c.outsourcing_keywords} (+${c.outsourcing_keywords * 25} pts)
                    </p>
                </div>

                <div style="margin-bottom:16px;">
                    <span style="color:var(--text-secondary); font-size:11px;">Detected Tech Stack</span><br>
                    <div style="margin-top:6px;">
                        ${c.tech_stack.map(t => `<span class="badge" style="background:rgba(99,102,241,0.2); color:#a5b4fc; margin-right:4px;">${t}</span>`).join('')}
                    </div>
                </div>

                <div style="display:flex; gap:10px;">
                    ${c.contact_url ? `<a href="${c.contact_url}" target="_blank" class="btn btn-secondary btn-sm"><i class="fa-solid fa-link"></i> Contact Page</a>` : ''}
                    ${c.linkedin_url ? `<a href="${c.linkedin_url}" target="_blank" class="btn btn-secondary btn-sm"><i class="fa-solid fa-brands fa-linkedin"></i> LinkedIn</a>` : ''}
                </div>
            `;

            document.getElementById('leadModal').classList.add('active');
        } catch (err) {
            console.error("Error opening modal:", err);
        }
    }

    document.getElementById('btnCloseModal').addEventListener('click', () => {
        document.getElementById('leadModal').classList.remove('active');
    });

    // Fetch Investors
    async function loadInvestors() {
        const search = document.getElementById('filterInvSearch').value;
        const itype = document.getElementById('filterInvType').value;
        const focus = document.getElementById('filterInvFocus').value;

        const params = new URLSearchParams({
            page: 1,
            limit: 50,
            investor_type: itype,
            focus: focus,
            search_query: search
        });

        try {
            const res = await fetch(`/api/investors?${params}`);
            const data = await res.json();
            const tbody = document.getElementById('investorTableBody');
            tbody.innerHTML = '';

            document.getElementById('investorCountBadge').innerText = `${data.investors.length} investors found`;

            data.investors.forEach(inv => {
                const tr = document.createElement('tr');
                const focusBadges = inv.focus.map(f => `<span class="badge" style="background:rgba(139,92,246,0.15); color:#c084fc; font-size:10px; margin-right:4px;">${f}</span>`).join('');
                const portBadges = inv.portfolio_highlights.map(p => `<span class="badge" style="background:rgba(255,255,255,0.08); font-size:10px; margin-right:4px;">${p}</span>`).join('');

                const copyEmailBtn = inv.public_email ? `<button class="btn-copy" onclick="copyToClipboard('${inv.public_email}', 'Investor Email')"><i class="fa-solid fa-copy"></i> Email</button>` : '';
                const copyPhoneBtn = inv.phone ? `<button class="btn-copy" onclick="copyToClipboard('${inv.phone}', 'Investor Phone')"><i class="fa-solid fa-copy"></i> Phone</button>` : '';

                tr.innerHTML = `
                    <td>
                        <strong>${inv.name}</strong><br>
                        <a href="${inv.website}" target="_blank" style="color:var(--accent-purple); text-decoration:none; font-size:11px;">
                            ${inv.website.replace('https://', '')} <i class="fa-solid fa-arrow-up-right-from-square" style="font-size:9px;"></i>
                        </a>
                    </td>
                    <td>
                        <span class="badge badge-high">${inv.investor_type}</span><br>
                        <small style="color:var(--text-secondary);">${inv.country} (${inv.city || 'Global'})</small>
                    </td>
                    <td>
                        <strong style="font-size:16px; color:#34d399;">${inv.score}</strong>
                        <span class="badge badge-high" style="font-size:10px;">${inv.priority_tier}</span>
                    </td>
                    <td><strong>${inv.check_size || 'N/A'}</strong></td>
                    <td>
                        ${inv.public_email ? `<span style="color:#a7f3d0;"><i class="fa-solid fa-envelope"></i> ${inv.public_email}</span>` : '<span style="color:var(--text-muted);">-</span>'}
                    </td>
                    <td>
                        <div style="display:flex; gap:4px; flex-direction:column;">
                            ${copyEmailBtn}
                            ${copyPhoneBtn}
                        </div>
                    </td>
                    <td>${focusBadges}</td>
                    <td>${portBadges}</td>
                `;
                tbody.appendChild(tr);
            });
        } catch (err) {
            console.error("Error loading investors:", err);
        }
    }

    // Run Automated Match for AGB Technologies
    document.getElementById('btnRunAGBMatch').addEventListener('click', async () => {
        try {
            const reqData = {
                company_name: "AGB Technologies",
                sectors: ["B2B SaaS", "AI", "Enterprise Software", "Automation"],
                target_market: ["India", "US", "UK"],
                funding_stage: "Seed",
                funding_amount: "$250K - $2M"
            };

            const res = await fetch('/api/investors/match', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(reqData)
            });

            const data = await res.json();
            const tbody = document.getElementById('investorTableBody');
            tbody.innerHTML = '';

            document.getElementById('investorCountBadge').innerText = `${data.matches.length} AI Ranked Matches for ${data.company_name}`;

            data.matches.forEach(m => {
                const inv = m.investor;
                const tr = document.createElement('tr');
                const focusBadges = inv.focus.map(f => `<span class="badge" style="background:rgba(139,92,246,0.15); color:#c084fc; font-size:10px; margin-right:4px;">${f}</span>`).join('');
                const reasonBadges = m.match_reasons.map(r => `<span class="badge badge-high" style="font-size:10px; margin-right:4px;"><i class="fa-solid fa-check"></i> ${r}</span>`).join('');

                const copyEmailBtn = inv.public_email ? `<button class="btn-copy" onclick="copyToClipboard('${inv.public_email}', 'Investor Email')"><i class="fa-solid fa-copy"></i> Email</button>` : '';
                const copyPhoneBtn = inv.phone ? `<button class="btn-copy" onclick="copyToClipboard('${inv.phone}', 'Investor Phone')"><i class="fa-solid fa-copy"></i> Phone</button>` : '';

                tr.innerHTML = `
                    <td>
                        <strong>${inv.name}</strong><br>
                        <a href="${inv.website}" target="_blank" style="color:var(--accent-purple); text-decoration:none; font-size:11px;">
                            ${inv.website.replace('https://', '')}
                        </a>
                    </td>
                    <td>
                        <span class="badge badge-high">${inv.investor_type}</span><br>
                        <small style="color:var(--text-secondary);">${inv.country}</small>
                    </td>
                    <td>
                        <div style="display:flex; align-items:center; gap:8px;">
                            <strong style="font-size:18px; color:#34d399;">${m.match_score}%</strong>
                            <span class="badge badge-high">Match</span>
                        </div>
                    </td>
                    <td><strong>${inv.check_size || 'N/A'}</strong></td>
                    <td>
                        ${inv.public_email ? `<span style="color:#a7f3d0;"><i class="fa-solid fa-envelope"></i> ${inv.public_email}</span>` : '<span style="color:var(--text-muted);">-</span>'}
                    </td>
                    <td>
                        <div style="display:flex; gap:4px; flex-direction:column;">
                            ${copyEmailBtn}
                            ${copyPhoneBtn}
                        </div>
                    </td>
                    <td>${focusBadges}</td>
                    <td>${reasonBadges}</td>
                `;
                tbody.appendChild(tr);
            });

            showToast("AI Investor Matching complete for AGB Technologies!");
        } catch (err) {
            console.error("Match error:", err);
        }
    });

    // Auto Seed Discovery Action
    document.getElementById('btnAutoSeed').addEventListener('click', async () => {
        try {
            const res = await fetch('/api/crawler/auto-seeds', { method: 'POST' });
            const data = await res.json();
            alert(`Live Discovery Engine synthesized ${data.count} new target company URLs across US/UK tech hubs.`);
            loadStats();
        } catch (err) {
            alert("Failed to run seed generator: " + err);
        }
    });

    // Export Excel (.xlsx) Actions
    document.getElementById('btnExportExcel').addEventListener('click', () => {
        const country = document.getElementById('filterCountry').value;
        const priority = document.getElementById('filterPriority').value;
        window.location.href = `/api/leads/export?country=${country}&priority=${priority}&page=1&limit=10000`;
    });

    document.getElementById('btnExportPeople').addEventListener('click', () => {
        window.location.href = `/api/people/export?page=1&limit=10000`;
    });

    document.getElementById('btnExportInvestors').addEventListener('click', () => {
        window.location.href = `/api/investors/export?page=1&limit=10000`;
    });

    // Toggle Crawler (Start / Stop)
    document.getElementById('btnToggleCrawler').addEventListener('click', async () => {
        const statusText = document.getElementById('crawlerStatusText').innerText;

        if (statusText.includes('IDLE')) {
            const res = await fetch('/api/crawler/start', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ seed_urls: [], mode: 'stealth' })
            });
            const data = await res.json();
            alert(`Continuous Crawl Daemon launched in Stealth Mode.`);
        } else {
            await fetch('/api/crawler/stop', { method: 'POST' });
            alert("Crawler stop signal sent.");
        }
        loadStats();
    });

    // Filter event listeners
    document.getElementById('filterSearch').addEventListener('input', () => { currentPage = 1; loadLeads(); });
    document.getElementById('filterCountry').addEventListener('change', () => { currentPage = 1; loadLeads(); });
    document.getElementById('filterPriority').addEventListener('change', () => { currentPage = 1; loadLeads(); });
    document.getElementById('filterHiringOnly').addEventListener('change', () => { currentPage = 1; loadLeads(); });
    document.getElementById('btnRefreshLeads').addEventListener('click', loadLeads);

    document.getElementById('filterPeopleSearch').addEventListener('input', loadPeople);
    document.getElementById('filterPeopleRole').addEventListener('change', loadPeople);

    document.getElementById('filterInvSearch').addEventListener('input', loadInvestors);
    document.getElementById('filterInvType').addEventListener('change', loadInvestors);
    document.getElementById('filterInvFocus').addEventListener('change', loadInvestors);

    // Proxy Loader
    async function loadProxies() {
        try {
            const res = await fetch('/api/proxies');
            const data = await res.json();
            const tbody = document.getElementById('proxyTableBody');
            tbody.innerHTML = '';

            data.proxies.forEach(p => {
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

    // Logs Loader
    async function loadLogs() {
        try {
            const res = await fetch('/api/logs');
            const data = await res.json();
            const consoleBox = document.getElementById('consoleLogs');
            consoleBox.innerHTML = '';

            data.logs.forEach(l => {
                const div = document.createElement('div');
                div.className = `log-line log-${l.level}`;
                div.innerText = `[${l.timestamp.slice(11, 19)}] [${l.level}] [${l.domain}] ${l.message}`;
                consoleBox.appendChild(div);
            });
        } catch (err) {
            console.error("Error loading logs:", err);
        }
    }

    // Initial Load & Refresh Interval
    loadStats();
    loadLeads();
    setInterval(loadStats, 4000);
    setInterval(loadLogs, 5000);
});
