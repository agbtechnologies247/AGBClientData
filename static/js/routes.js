export function initRoutes(onTabChange) {
    const navItems = document.querySelectorAll('.nav-item[data-tab]');
    const tabContents = document.querySelectorAll('.tab-content');
    const pageHeading = document.getElementById('pageHeading');

    const headings = {
        tabDashboard: 'Target Companies & Lead Intelligence',
        tabPipeline: 'Multi-Stage Leads Qualification Pipeline',
        tabPeople: 'Verified Decision Makers & Executive Contacts',
        tabInvestors: 'Verified B2B SaaS & AI Investors',
        tabOutreach: 'Sent Email History & Delivery Status',
        tabCrawler: 'Anti-Blocking Stealth Crawler Config',
        tabProxies: 'Anti-Blocking Proxy Rotation Pool',
        tabLogs: 'Live Crawler Console Logs'
    };

    navItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            const tabId = item.getAttribute('data-tab');

            navItems.forEach(n => n.classList.remove('active'));
            tabContents.forEach(c => c.classList.remove('active'));

            item.classList.add('active');
            const targetTab = document.getElementById(tabId);
            if (targetTab) targetTab.classList.add('active');

            if (pageHeading && headings[tabId]) {
                pageHeading.innerText = headings[tabId];
            }

            if (typeof onTabChange === 'function') {
                onTabChange(tabId);
            }
        });
    });

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
