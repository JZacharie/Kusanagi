/**
 * Kusanagi Sidebar Navigation
 * Responsive sidebar with hamburger toggle
 */

class Sidebar {
  constructor() {
    this.sidebar = document.querySelector('.sidebar');
    this.toggle = document.querySelector('.sidebar-toggle');
    this.overlay = document.querySelector('.sidebar-overlay');
    this.isMobile = window.innerWidth <= 768;
    this.isCollapsed = localStorage.getItem('sidebar-collapsed') === 'true';
    
    this.init();
  }
  
  init() {
    if (!this.sidebar) return;
    
    // Check if we were on a specific page
    const currentPage = localStorage.getItem('current-page') || 'dashboard';
    this.setActivePage(currentPage);
    
    // Event listeners
    if (this.toggle) {
      this.toggle.addEventListener('click', (e) => this.handleToggle(e));
    }
    
    if (this.overlay) {
      this.overlay.addEventListener('click', () => this.close());
    }
    
    // Navigation links
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
      this.sidebar.classList.toggle('open');
      this.toggle.classList.toggle('active');
      if (this.overlay) {
        this.overlay.classList.toggle('active');
      }
      document.body.style.overflow = this.sidebar.classList.contains('open') ? 'hidden' : '';
    } else {
      // Desktop: toggle collapsed state
      this.isCollapsed = !this.isCollapsed;
      this.sidebar.classList.toggle('collapsed', this.isCollapsed);
      this.toggle.classList.toggle('active', this.isCollapsed);
      localStorage.setItem('sidebar-collapsed', this.isCollapsed);
      
      // Update toggle position
      const newLeft = this.isCollapsed ? '70px' : `calc(var(--sidebar-width) + 10px)`;
      this.toggle.style.left = newLeft;
    }
  }
  
  close() {
    if (this.isMobile) {
      this.sidebar.classList.remove('open');
      this.toggle.classList.remove('active');
      if (this.overlay) {
        this.overlay.classList.remove('active');
      }
      document.body.style.overflow = '';
    }
  }
  
  open() {
    if (this.isMobile) {
      this.sidebar.classList.add('open');
      this.toggle.classList.add('active');
      if (this.overlay) {
        this.overlay.classList.add('active');
      }
      document.body.style.overflow = 'hidden';
    }
  }
  
  handleNavClick(e, page) {
    e.preventDefault();
    
    // Close mobile sidebar after navigation
    if (this.isMobile) {
      this.close();
    }
    
    // Load the page content
    this.loadPage(page);
    
    // Update URL hash without triggering scroll
    history.pushState(null, null, `#${page}`);
  }
  
  loadPage(page) {
    // Set active state
    this.setActivePage(page);
    
    // Store current page
    localStorage.setItem('current-page', page);
    
    // Update page title
    const titles = {
      'dashboard': 'Dashboard',
      'pods': 'Pods',
      'nodes': 'Nodes',
      'services': 'Services',
      'ingress': 'Ingress',
      'events': 'Events',
      'storage': 'Storage',
      'backups': 'Backups',
      'security': 'Security',
      'alerts': 'Alerts',
      'chat': 'AI Chat',
      'settings': 'Settings'
    };
    
    document.title = `Kusanagi - ${titles[page] || 'Dashboard'}`;
    
    // Update header title
    const headerTitle = document.querySelector('.header-title');
    if (headerTitle) {
      headerTitle.textContent = titles[page] || 'Dashboard';
    }
    
    // Load content (placeholder - actual content loading would be here)
    this.updateContent(page);
    
    // Trigger custom event
    window.dispatchEvent(new CustomEvent('pagechange', { detail: { page } }));
  }
  
  setActivePage(page) {
    document.querySelectorAll('.nav-link').forEach(link => {
      link.classList.remove('active');
      if (link.dataset.page === page) {
        link.classList.add('active');
      }
    });
  }
  
  updateContent(page) {
    // Show loading state
    const contentBody = document.querySelector('.content-body');
    if (!contentBody) return;
    
    contentBody.style.opacity = '0.5';
    
    // Simulate page load (in real implementation, fetch actual content)
    setTimeout(() => {
      // Here you would typically:
      // 1. Fetch new content via AJAX
      // 2. Update the DOM
      // 3. Re-initialize any page-specific JS
      
      // For now, we just restore opacity
      contentBody.style.opacity = '1';
      
      // Scroll to top
      contentBody.scrollTop = 0;
      window.scrollTo(0, 0);
    }, 150);
  }
  
  handleKeydown(e) {
    // ESC to close sidebar on mobile
    if (e.key === 'Escape' && this.isMobile) {
      this.close();
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
      this.toggle.classList.remove('active');
      if (this.overlay) {
        this.overlay.classList.remove('active');
      }
      document.body.style.overflow = '';
    } else {
      // Desktop: use saved collapsed state
      this.sidebar.classList.toggle('collapsed', this.isCollapsed);
      this.toggle.classList.toggle('active', this.isCollapsed);
      if (this.overlay) {
        this.overlay.classList.remove('active');
      }
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
          this.open();
        }
        // Swipe left to close
        else if (diff < -swipeThreshold && this.sidebar.classList.contains('open')) {
          this.close();
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
  
  // Handle initial hash
  const hash = window.location.hash.slice(1);
  if (hash) {
    window.sidebar.loadPage(hash);
  }
  
  // Handle back/forward buttons
  window.addEventListener('popstate', () => {
    const page = window.location.hash.slice(1) || 'dashboard';
    window.sidebar.loadPage(page);
  });
});

// Export for module use if needed
if (typeof module !== 'undefined' && module.exports) {
  module.exports = Sidebar;
}
