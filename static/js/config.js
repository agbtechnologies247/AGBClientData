export const API_ENDPOINTS = {
    STATS: '/api/stats',
    LEADS: '/api/leads',
    PEOPLE: '/api/people',
    INVESTORS: '/api/investors',
    INVESTORS_MATCH: '/api/investors/match',
    CAMPAIGNS: '/api/campaigns',
    OUTREACH_TRIGGER: '/api/outreach/trigger',
    OUTREACH_HISTORY: '/api/outreach/history',
    CRAWLER_START: '/api/crawler/start',
    CRAWLER_STOP: '/api/crawler/stop',
    CRAWLER_CLEAR_DB: '/api/crawler/clear-database',
    PROXIES: '/api/proxies',
    LOGS: '/api/logs',
    LOGS_CLEAR: '/api/logs/clear',
    EXPORT_LEADS: '/api/leads/export',
    EXPORT_PEOPLE: '/api/people/export',
    EXPORT_INVESTORS: '/api/investors/export'
};

export const PIPELINE_STAGES = [
    { id: 'DISCOVERED', name: 'Discovered Target', badgeClass: 'badge-low' },
    { id: 'ENRICHED', name: 'Enriched & Verified', badgeClass: 'badge-medium' },
    { id: 'CONTACTED', name: 'Outreach Contacted', badgeClass: 'badge-sent' },
    { id: 'QUALIFIED', name: 'Qualified Lead', badgeClass: 'badge-high' },
    { id: 'PROPOSAL', name: 'Proposal Sent', badgeClass: 'badge-high' },
    { id: 'CANCELLED', name: 'Cancelled ✖', badgeClass: 'badge-cancelled' },
    { id: 'REJECTED', name: 'Rejected ✖', badgeClass: 'badge-rejected' },
    { id: 'WON', name: 'Deal Won ✓', badgeClass: 'badge-high' }
];
