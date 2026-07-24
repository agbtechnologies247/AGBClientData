import { API_ENDPOINTS, PIPELINE_STAGES } from '../config.js';

export async function loadPipeline() {
    try {
        const res = await fetch(`${API_ENDPOINTS.LEADS}?limit=100`);
        const data = await res.json();
        const container = document.getElementById('pipelineContainer');
        if (!container) return;
        container.innerHTML = '';

        PIPELINE_STAGES.forEach(st => {
            const colLeads = (data.leads || []).filter(l => (l.qualification_stage || 'DISCOVERED') === st.id);
            const col = document.createElement('div');
            col.className = 'pipeline-col';
            col.innerHTML = `
                <div class="pipeline-col-header">
                    <span>${st.name}</span>
                    <span class="badge ${st.badgeClass}">${colLeads.length}</span>
                </div>
                <div style="flex:1;">
                    ${colLeads.map(l => `
                        <div class="pipeline-card">
                            <strong>${l.name}</strong>
                            <span>${l.domain}</span>
                            <span>${l.contact_person || 'Executive Leadership'}</span>
                            <select class="pipeline-select" onchange="updateLeadStage(${l.id}, this.value)">
                                ${PIPELINE_STAGES.map(s => `<option value="${s.id}" ${s.id === st.id ? 'selected' : ''}>Move to: ${s.name}</option>`).join('')}
                            </select>
                        </div>
                    `).join('')}
                </div>
            `;
            container.appendChild(col);
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
            if (window.showToast) window.showToast(`Lead moved to stage: ${stage}`);
            loadPipeline();
        }
    } catch (err) {
        console.error("Error updating lead stage:", err);
    }
};
