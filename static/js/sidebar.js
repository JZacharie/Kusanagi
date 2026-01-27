/**
 * Kusanagi Sidebar Manager
 * Handles sidebar toggling and accessibility states
 */

const SidebarManager = {
    isOpen: true,

    init() {
        this.setupEventListeners();
        this.restoreState();
        console.log('✅ Sidebar Manager initialized');
    },

    setupEventListeners() {
        const toggleBtn = document.getElementById('sidebar-toggle');
        if (toggleBtn) {
            toggleBtn.addEventListener('click', () => this.toggle());
        }

        // Close sidebar on mobile when clicking outside
        document.addEventListener('click', (e) => {
            const sidebar = document.getElementById('sidebar');
            const toggleBtn = document.getElementById('sidebar-toggle');

            if (window.innerWidth <= 768 &&
                sidebar &&
                !sidebar.contains(e.target) &&
                !toggleBtn.contains(e.target) &&
                this.isOpen) {
                this.toggle(false); // Force close
            }
        });
    },

    toggle(forceState = null) {
        const sidebar = document.getElementById('sidebar');
        const content = document.getElementById('main-content');
        const toggleBtn = document.getElementById('sidebar-toggle');

        if (!sidebar || !content) return;

        // Determine new state
        const newState = forceState !== null ? forceState : !this.isOpen;
        this.isOpen = newState;

        // Update classes
        if (this.isOpen) {
            sidebar.classList.remove('collapsed');
            content.classList.remove('expanded');
        } else {
            sidebar.classList.add('collapsed');
            content.classList.add('expanded');
        }

        // Update ARIA attributes
        if (toggleBtn) {
            toggleBtn.setAttribute('aria-expanded', this.isOpen);
            toggleBtn.setAttribute('aria-label', this.isOpen ? 'Close Sidebar' : 'Open Sidebar');
        }

        sidebar.setAttribute('aria-hidden', !this.isOpen);

        // Save state
        localStorage.setItem('kusanagi_sidebar_open', this.isOpen);
    },

    restoreState() {
        // Default to open on desktop, closed on mobile
        const isMobile = window.innerWidth <= 768;
        const savedState = localStorage.getItem('kusanagi_sidebar_open');

        let shouldBeOpen = true;

        if (savedState !== null) {
            shouldBeOpen = savedState === 'true';
        } else if (isMobile) {
            shouldBeOpen = false;
        }

        this.toggle(shouldBeOpen);
    }
};

// Global accessor for legacy inline calls if any
window.toggleSidebar = () => SidebarManager.toggle();

// Auto-init
document.addEventListener('DOMContentLoaded', () => {
    SidebarManager.init();
});
