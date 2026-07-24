import { API_ENDPOINTS, PIPELINE_STAGES } from '../config.js';

let pipelinePage = 1;
const pipelineLimit = 10;

export async function loadPipeline(newPage) {
    if (newPage) pipelinePage = newPage;

    try {
        const offset = (pipelinePage - 1) * pipelineLimit;
        const res = await fetch(`${API_ENDPOINTS.LEADS}?page=${pipelinePage}&limit=100`);
        const data = await res.json();
        const container = document.getElementById('pipelineContainer');
        if (!container) return;
        container.innerHTML = '';

        const allLeads = data.leads || [];
        const total = data.total || allLeads.length;

        PIPELINE_STAGES.forEach(st => {
            const stageLeads = allLeads.filter(l => (l.qualification_stage || 'DISCOVERED') === st.id);
            // Take 10 unique & latest targets for this stage page
            const displayLeads = stageLeads.slice(0, 10);

            const col = document.createElement('div');
            col.className = 'pipeline-col';
            col.innerHTML = `
                <div class="pipeline-col-header">
                    <span>${st.name}</span>
                    <span class="badge ${st.badgeClass}">${stageLeads.length}</span>
                </div>
                <div style="flex:1;">
                    ${displayLeads.length > 0 ? displayLeads.map(l => `
                        <div class="pipeline-card">
                            <strong style="color:var(--text-primary); font-size:14px;">${l.name}</strong>
                            <span style="color:var(--text-secondary); font-size:12px; margin-top:2px;">${l.domain}</span>
                            <span style="color:var(--accent); font-size:12px; font-weight:600; margin-top:4px;">${l.contact_person || 'Executive Leadership'}</span>
                            <select class="pipeline-select" style="margin-top:8px; width:100%; padding:6px 10px; border-radius:6px; border:1px solid var(--border-color);" onchange="updateLeadStage(${l.id}, this.value)">
                                ${PIPELINE_STAGES.map(s => `<option value="${s.id}" ${s.id === st.id ? 'selected' : ''}>${s.name}</option>`).join('')}
                            </select>
                        </div>
                    `).join('') : '<div style="padding:16px; text-align:center; color:var(--text-muted); font-size:12px;">No targets in stage</div>'}
                </div>
            `;
            container.appendChild(col);
        });

        // Pipeline Pagination Controls Footer
        const totalPages = Math.max(1, Math.ceil(total / 100));
        let footer = document.getElementById('pipelinePaginationFooter');
        if (!footer) {
            footer = document.createElement('div');
            footer.id = 'pipelinePaginationFooter';
            footer.className = 'pagination-bar card';
            footer.style.marginTop = '20px';
            container.parentNode.appendChild(footer);
        }

        footer.innerHTML = `
            <div class="pagination-info">Showing batch <strong>${pipelinePage}</strong> (10 unique targets per stage column)</div>
            <div class="pagination-controls">
                <button class="btn btn-secondary btn-sm" id="btnPrevPipeline" ${pipelinePage <= 1 ? 'disabled' : ''}><i class="fa-solid fa-chevron-left"></i> Previous 10</button>
                <span class="page-indicator">Page ${pipelinePage}</span>
                <button class="btn btn-primary btn-sm" id="btnNextPipeline"><i class="fa-solid fa-rotate-right"></i> Next 10 Targets</button>
            </div>
        `;

        document.getElementById('btnPrevPipeline')?.addEventListener('click', () => {
            if (pipelinePage > 1) loadPipeline(pipelinePage - 1);
        });

        document.getElementById('btnNextPipeline')?.addEventListener('click', () => {
            loadPipeline(pipelinePage + 1);
        });

    } catch (err) {
        console.error("Error loading pipeline:", err);
    }
}

window.updateLeadStage = async function(id, stage) {
    try {
        const res = await fetch(`${API_ENDPOINTS.LEADS}/${id}/stage`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ stage })
        });
        if (res.ok) {
            if (window.showToast) window.showToast(`Lead stage updated to ${stage}`);
            loadPipeline();
        }
    } catch (err) {
        console.error("Error updating lead stage:", err);
    }
};
