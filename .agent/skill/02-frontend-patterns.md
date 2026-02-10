# Frontend Patterns

## Architecture
Vanilla JS with module pattern, no frameworks.

## Module Structure
```javascript
const ModuleName = {
    // State
    data: [],
    lastFetch: 0,
    TTL: 180000, // 3 minutes

    // Init
    init() {
        this.fetchData();
        setInterval(() => this.fetchAll(), 30000);
        this.setupFocusRefresh();
    },

    // Fetch with cache logic
    async fetchData() {
        const now = Date.now();
        if (this.lastFetch !== 0) {
            const activeTab = window.KusanagiDashboard?.activeTab;
            if (activeTab !== 'tabName') return;
            if (now - this.lastFetch < this.TTL) return;
        }
        this.lastFetch = now;
        // API call...
    },

    // Focus refresh pattern
    setupFocusRefresh() {
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                // Check TTL and refresh
            }
        });
    }
};
```

## Key Modules

| Module | Purpose | Key Methods |
|--------|---------|-------------|
| `K8sManager` | K8s operations | fetchServices(), fetchIngress() |
| `DashboardManager` | Core dashboard | switchTab(), refreshAll() |
| `NewsManager` | News feed | fetchNews(), manualRefresh() |
| `WeatherDashboard` | Weather widget | fetchAndRender() |

## API Client Pattern
```javascript
const response = await fetch('/api/endpoint');
const data = await response.json();
// Always expect JSON, handle empty arrays
```

## Tab System
```javascript
// Tabs defined in sidebar.js
const tabs = {
    'dashboard': 'Dashboard',
    'services': 'Services',
    'ingress': 'Ingress',
    'weather': 'Weather',
    // ...
};
```
