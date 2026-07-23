/**
 * SOLID API Client - Single Responsibility for HTTP Communication
 */
export class ApiClient {
    static async getStats() {
        const res = await fetch('/api/stats');
        return await res.json();
    }

    static async getLeads(params) {
        const res = await fetch(`/api/leads?${new URLSearchParams(params)}`);
        return await res.json();
    }

    static async getLeadDetail(id) {
        const res = await fetch(`/api/leads/${id}`);
        return await res.json();
    }

    static async validateLead(id) {
        const res = await fetch(`/api/leads/${id}/validate`, { method: 'POST' });
        return await res.json();
    }

    static async getPeople(params) {
        const res = await fetch(`/api/people?${new URLSearchParams(params)}`);
        return await res.json();
    }

    static async getInvestors(params) {
        const res = await fetch(`/api/investors?${new URLSearchParams(params)}`);
        return await res.json();
    }

    static async matchInvestors(payload) {
        const res = await fetch('/api/investors/match', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        return await res.json();
    }

    static async getOutreachHistory(params) {
        const res = await fetch(`/api/outreach/history?${new URLSearchParams(params)}`);
        return await res.json();
    }

    static async triggerOutreachBatch() {
        const res = await fetch('/api/outreach/trigger', { method: 'POST' });
        return await res.json();
    }

    static async startCrawler(seedUrls = [], mode = 'stealth') {
        const res = await fetch('/api/crawler/start', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ seed_urls: seedUrls, mode: mode })
        });
        return await res.json();
    }

    static async stopCrawler() {
        const res = await fetch('/api/crawler/stop', { method: 'POST' });
        return await res.json();
    }

    static async getProxies() {
        const res = await fetch('/api/proxies');
        return await res.json();
    }

    static async addProxies(proxies) {
        const res = await fetch('/api/proxies', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ proxies: proxies })
        });
        return await res.json();
    }

    static async getLogs() {
        const res = await fetch('/api/logs');
        return await res.json();
    }

    static async clearDatabase() {
        const res = await fetch('/api/database/clear', { method: 'POST' });
        return await res.json();
    }
}
