/**
 * Kusanagi Sidebar Navigation
 * Responsive sidebar with hamburger toggle - SPA mode (no page reload)
 */

class Sidebar {
  constructor() {
    this.sidebar = document.querySelector('.sidebar');
    this.toggle = document.querySelector('.header-toggle, .sidebar-toggle');
    this.overlay = document.querySelector('.sidebar-overlay');
    // Defer isMobile check to avoid forced reflow during construction
    this._isMobile = null;
    this.isCollapsed = localStorage.getItem('sidebar-collapsed') === 'true';

    // Defer init to next frame to avoid blocking initial render
    requestAnimationFrame(() => this.init());
  }

  get isMobile() {
    if (this._isMobile === null) {
      this._isMobile = window.innerWidth <= 768;
    }
    return this._isMobile;
  }

  set isMobile(value) {
    this._isMobile = value;
  }

  init() {
    if (!this.sidebar) return;

    // Get current tab from URL hash or default to dashboard
    const currentTab = window.location.hash.slice(1) || 'argocd';
    this.setActiveTab(currentTab);

    // Event listeners
    if (this.toggle) {
      this.toggle.addEventListener('click', (e) => this.handleToggle(e));
    }

    if (this.overlay) {
      this.overlay.addEventListener('click', () => this.closeMobile());
    }

    // Navigation links - use switchTab instead of page navigation
    document.querySelectorAll('.nav-link[data-page]').forEach(link => {
      link.addEventListener('click', (e) => this.handleNavClick(e, link.dataset.page));
    });

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => this.handleKeydown(e));

    // Window resize
    window.addEventListener('resize', () => this.handleResize());

    // Initial state
    this.updateState();

    // Swipe gesture for mobile
    this.initSwipeGesture();
  }

  handleToggle(e) {
    e.preventDefault();
    e.stopPropagation();

    if (this.isMobile) {
      // Mobile: toggle open/close with overlay
      if (this.sidebar.classList.contains('open')) {
        this.closeMobile();
      } else {
        this.openMobile();
      }
    } else {
      // Desktop: toggle collapsed state
      this.isCollapsed = !this.isCollapsed;
      this.sidebar.classList.toggle('collapsed', this.isCollapsed);
      localStorage.setItem('sidebar-collapsed', this.isCollapsed);
    }
  }

  openMobile() {
    this.sidebar.classList.add('open');
    if (this.overlay) {
      this.overlay.classList.add('active');
    }
    document.body.style.overflow = 'hidden';
  }

  closeMobile() {
    this.sidebar.classList.remove('open');
    if (this.overlay) {
      this.overlay.classList.remove('active');
    }
    document.body.style.overflow = '';
  }

  handleNavClick(e, tabName) {
    e.preventDefault();

    // Batch all UI updates in single animation frame
    requestAnimationFrame(() => {
      // Close mobile sidebar after navigation
      if (this.isMobile) {
        this.closeMobile();
      }

      // Use existing switchTab function
      if (typeof switchTab === 'function') {
        switchTab(tabName);
      } else {
        console.warn('switchTab function not found');
      }

      // Update active state
      this.setActiveTab(tabName);
    });

    // Update URL hash without triggering scroll
    history.pushState(null, null, `#${tabName}`);
  }

  setActiveTab(tabName) {
    // Batch DOM reads and writes to avoid forced reflow
    const links = document.querySelectorAll('.nav-link');
    const activeLink = document.querySelector(`.nav-link[data-page="${tabName}"]`);
    const headerTitle = document.querySelector('.header-title');

    // Write phase - batch all DOM writes
    links.forEach(link => link.classList.remove('active'));
    if (activeLink) activeLink.classList.add('active');

    if (headerTitle) {
      const titles = {
        'argocd': 'ArgoCD', 'system': 'System', 'proxmox': 'Proxmox',
        'alerts': 'Alerts', 'events': 'Events', 'nodes': 'Nodes',
        'pods': 'Pods', 'services': 'Services', 'ingress': 'Ingress',
        'storage': 'Storage', 'metrics': 'Metrics', 'network': 'Network',
        'backups': 'Backups', 'security': 'Security',
        'homeassistant': 'Home Assistant', 'mqtt': 'MQTT', 'calendar': 'Calendar',
        'weather': 'Weather', 'chat': 'AI Chat', 'docs': 'About',
        'news': 'News'
      };
      headerTitle.textContent = titles[tabName] || tabName.charAt(0).toUpperCase() + tabName.slice(1);
    }
  }

  handleKeydown(e) {
    // ESC to close sidebar on mobile
    if (e.key === 'Escape' && this.isMobile && this.sidebar.classList.contains('open')) {
      this.closeMobile();
    }

    // Alt+S to toggle sidebar
    if (e.altKey && e.key === 's') {
      e.preventDefault();
      this.handleToggle(e);
    }
  }

  handleResize() {
    const wasMobile = this.isMobile;
    this.isMobile = window.innerWidth <= 768;

    if (wasMobile !== this.isMobile) {
      // Reset state when switching between mobile/desktop
      this.updateState();
    }
  }

  updateState() {
    if (this.isMobile) {
      // Mobile: sidebar starts closed
      this.sidebar.classList.remove('collapsed');
      this.sidebar.classList.remove('open');
      if (this.overlay) {
        this.overlay.classList.remove('active');
      }
      document.body.style.overflow = '';
    } else {
      // Desktop: use saved collapsed state
      this.sidebar.classList.toggle('collapsed', this.isCollapsed);
    }
  }

  initSwipeGesture() {
    let touchStartX = 0;
    let touchEndX = 0;
    const swipeThreshold = 50;

    // Swipe from left edge to open
    document.addEventListener('touchstart', (e) => {
      touchStartX = e.changedTouches[0].screenX;
    }, { passive: true });

    document.addEventListener('touchend', (e) => {
      touchEndX = e.changedTouches[0].screenX;
      const diff = touchEndX - touchStartX;

      if (this.isMobile) {
        // Swipe right from left edge (within 50px) to open
        if (diff > swipeThreshold && touchStartX < 50 && !this.sidebar.classList.contains('open')) {
          this.openMobile();
        }
        // Swipe left to close
        else if (diff < -swipeThreshold && this.sidebar.classList.contains('open')) {
          this.closeMobile();
        }
      }
    }, { passive: true });
  }

  // Public API
  collapse() {
    if (!this.isMobile) {
      this.isCollapsed = true;
      this.sidebar.classList.add('collapsed');
      localStorage.setItem('sidebar-collapsed', 'true');
    }
  }

  expand() {
    if (!this.isMobile) {
      this.isCollapsed = false;
      this.sidebar.classList.remove('collapsed');
      localStorage.setItem('sidebar-collapsed', 'false');
    }
  }

  isOpen() {
    return this.isMobile
      ? this.sidebar.classList.contains('open')
      : !this.sidebar.classList.contains('collapsed');
  }
}

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', () => {
  window.sidebar = new Sidebar();

  // Apply feature flags
  if (window.KUSANAGI_FULL_FEATURES === false) {
    const restrictedPages = ['news', 'streaming', 'network', 'homeassistant', 'mqtt', 'weather', 'proxmox', 'system', 'docs'];
    restrictedPages.forEach(page => {
      const navItem = document.querySelector(`.nav-link[data-page="${page}"]`);
      if (navItem && navItem.closest('.nav-item')) {
        navItem.closest('.nav-item').style.display = 'none';
      }
      const section = document.querySelector(`section.tab-content[data-tab="${page}"]`);
      if (section) {
        section.remove();
      }
    });
    // Hide "Refresh Site" button
    const refreshLink = document.querySelector('a[onclick*="clearKusanagiCache"]');
    if (refreshLink && refreshLink.closest('.nav-item')) {
      refreshLink.closest('.nav-item').style.display = 'none';
    }
    // Hide empty nav-sections (e.g. Integrations when all items are hidden)
    document.querySelectorAll('.nav-section').forEach(section => {
      const visibleItems = section.querySelectorAll('.nav-item:not([style*="display: none"])');
      if (visibleItems.length === 0) {
        section.style.display = 'none';
      }
    });
  }

  // Initialize dashboard header elements visibility
  const dashboardHeader = document.getElementById("dashboard-header-elements");
  const hash = window.location.hash.slice(1);
  if (dashboardHeader) {
    // Show only on ArgoCD page (default if no hash)
    dashboardHeader.style.display = (hash === "" || hash === "argocd") ? "block" : "none";
  }

  // Handle initial hash
  if (hash && typeof switchTab === 'function') {
    // Wait for dashboard.js to initialize
    setTimeout(() => {
      switchTab(hash);
      window.sidebar.setActiveTab(hash);
    }, 100);
  }

  // Handle back/forward buttons
  window.addEventListener('popstate', () => {
    const tab = window.location.hash.slice(1) || 'argocd';
    if (typeof switchTab === 'function') {
      switchTab(tab);
      window.sidebar.setActiveTab(tab);
    }
  });
});

// Export for module use if needed
if (typeof module !== 'undefined' && module.exports) {
  module.exports = Sidebar;
}
