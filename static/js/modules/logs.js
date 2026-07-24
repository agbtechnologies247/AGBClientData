import { API_ENDPOINTS } from '../config.js';

export async function loadLogs() {
    try {
        const res = await fetch(API_ENDPOINTS.LOGS);
        const data = await res.json();
        const consoleBox = document.getElementById('consoleLogs');
        if (!consoleBox) return;
        consoleBox.innerHTML = '';

        (data.logs || []).forEach(l => {
            const div = document.createElement('div');
            div.className = `log-line log-${l.level}`;
            div.innerText = `[${l.timestamp.slice(11, 19)}] [${l.level}] [${l.domain}] ${l.message}`;
            consoleBox.appendChild(div);
        });
    } catch (err) {
        console.error("Error loading logs:", err);
    }
}

export function initLogsHandlers(showToast) {
    const btnClearLogs = document.getElementById('btnClearLogs');
    if (btnClearLogs) {
        btnClearLogs.addEventListener('click', async () => {
            try {
                const res = await fetch(API_ENDPOINTS.LOGS_CLEAR, { method: 'POST' });
                if (res.ok) {
                    const consoleBox = document.getElementById('consoleLogs');
                    if (consoleBox) consoleBox.innerHTML = '';
                    if (showToast) showToast("Crawler console logs cleared!");
                }
            } catch (err) {
                console.error("Error clearing logs:", err);
            }
        });
    }
}
