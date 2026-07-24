export function initRoutes(onTabChange) {
    const navItems = document.querySelectorAll('.nav-item[data-tab]');
    const tabContents = document.querySelectorAll('.tab-content');
    const pageHeading = document.getElementById('pageHeading');

    const headings = {
        tabDashboard: 'Target Companies & Lead Intelligence',
        tabPipeline: 'Multi-Stage Leads Qualification Pipeline',
        tabPeople: 'Verified Decision Makers & Executive Contacts',
        tabInvestors: 'Verified B2B SaaS & AI Investors',
        tabCampaigns: 'Sent Email History & Delivery Status',
        tabCrawler: 'Anti-Blocking Stealth Crawler Config',
        tabProxies: 'Anti-Blocking Proxy Rotation Pool',
        tabLogs: 'Live Crawler Console Logs'
    };

    const pathToTabMap = {
        '/': 'tabDashboard',
        '/dashboard': 'tabDashboard',
        '/leads': 'tabDashboard',
        '/pipeline': 'tabPipeline',
        '/investors': 'tabInvestors',
        '/people': 'tabPeople',
        '/campaigns': 'tabCampaigns',
        '/outreach': 'tabCampaigns',
        '/crawler': 'tabCrawler',
        '/proxies': 'tabProxies',
        '/logs': 'tabLogs'
    };

    const tabToPathMap = {
        tabDashboard: '/leads',
        tabPipeline: '/pipeline',
        tabInvestors: '/investors',
        tabPeople: '/people',
        tabCampaigns: '/campaigns',
        tabCrawler: '/crawler',
        tabProxies: '/proxies',
        tabLogs: '/logs'
    };

    function activateTab(tabId, pushState = true) {
        const normalizedTabId = tabId.startsWith('tab') ? tabId : `tab${tabId.charAt(0).toUpperCase() + tabId.slice(1)}`;

        navItems.forEach(n => {
            const raw = n.getAttribute('data-tab') || 'dashboard';
            const tid = raw.startsWith('tab') ? raw : `tab${raw.charAt(0).toUpperCase() + raw.slice(1)}`;
            if (tid === normalizedTabId) {
                n.classList.add('active');
            } else {
                n.classList.remove('active');
            }
        });

        tabContents.forEach(c => c.classList.remove('active'));

        const targetTab = document.getElementById(normalizedTabId);
        if (targetTab) {
            targetTab.classList.add('active');
        }

        if (pageHeading && headings[normalizedTabId]) {
            pageHeading.innerText = headings[normalizedTabId];
        }

        if (pushState && tabToPathMap[normalizedTabId]) {
            const targetPath = tabToPathMap[normalizedTabId];
            if (window.location.pathname !== targetPath) {
                history.pushState({ tabId: normalizedTabId }, '', targetPath);
            }
        }

        if (typeof onTabChange === 'function') {
            onTabChange(normalizedTabId);
        }
    }

    navItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            const rawTabId = item.getAttribute('data-tab') || 'dashboard';
            activateTab(rawTabId, true);
        });
    });

    // Handle Browser Back / Forward and Initial URL Path
    window.addEventListener('popstate', () => {
        const currentPath = window.location.pathname.toLowerCase();
        const tabId = pathToTabMap[currentPath] || 'tabDashboard';
        activateTab(tabId, false);
    });

    // Activate initial tab based on current URL path
    const initialPath = window.location.pathname.toLowerCase();
    const initialTabId = pathToTabMap[initialPath] || 'tabDashboard';
    activateTab(initialTabId, false);

    // Sidebar Collapsible Toggle & Persistence
    const btnToggleSidebar = document.getElementById('btnToggleSidebar');
    const appLayout = document.querySelector('.app-layout');

    if (localStorage.getItem('sidebar_collapsed') === 'true' && appLayout) {
        appLayout.classList.add('sidebar-collapsed');
    }

    if (btnToggleSidebar && appLayout) {
        btnToggleSidebar.addEventListener('click', () => {
            appLayout.classList.toggle('sidebar-collapsed');
            const isCollapsed = appLayout.classList.contains('sidebar-collapsed');
            localStorage.setItem('sidebar_collapsed', isCollapsed ? 'true' : 'false');
        });
    }
}
