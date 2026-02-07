/**
 * KUSANAGI Core Frontend Logic
 * Handles WebSocket, Tab Navigation, and Table Management
 */

// === WEBSOCKET NOTIFICATIONS ===
let wsConnection = null;
let wsReconnectAttempts = 0;
const WS_MAX_RECONNECT_ATTEMPTS = 5;
const WS_BASE_RECONNECT_DELAY = 1000;

function initWebSocket() {
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/api/ws/notifications`;

    updateWsStatus('connecting');

    try {
        if (wsConnection && wsConnection.readyState === WebSocket.OPEN) {
            wsConnection.close();
        }
        wsConnection = new WebSocket(wsUrl);

        wsConnection.onopen = function () {
            console.log('✅ WebSocket connected');
            wsReconnectAttempts = 0;
            updateWsStatus('connected');
        };

        wsConnection.onmessage = function (event) {
            try {
                const data = JSON.parse(event.data);
                handleWsMessage(data);
            } catch (e) {
                console.error('Failed to parse WebSocket message:', e);
            }
        };

        wsConnection.onclose = function (event) {
            console.log('WebSocket closed:', event.code, event.reason);
            updateWsStatus('disconnected');
            attemptReconnect();
        };

        wsConnection.onerror = function (error) {
            console.error('WebSocket error:', error);
            updateWsStatus('disconnected');
        };
    } catch (error) {
        console.error('Failed to create WebSocket:', error);
        updateWsStatus('disconnected');
    }
}

function attemptReconnect() {
    if (wsReconnectAttempts < WS_MAX_RECONNECT_ATTEMPTS) {
        wsReconnectAttempts++;
        const delay = WS_BASE_RECONNECT_DELAY * Math.pow(2, wsReconnectAttempts - 1);
        console.log(`Attempting WebSocket reconnect (${wsReconnectAttempts}/${WS_MAX_RECONNECT_ATTEMPTS}) in ${delay}ms...`);
        setTimeout(initWebSocket, delay);
    } else {
        console.log('Max WebSocket reconnect attempts reached');
    }
}

function updateWsStatus(status) {
    const indicator = document.getElementById('ws-indicator');
    const statusEl = document.getElementById('ws-status');
    if (!indicator || !statusEl) return;

    indicator.className = 'ws-indicator ' + status;

    const labels = {
        'connected': 'Live',
        'disconnected': 'Offline',
        'connecting': 'Connecting...'
    };

    statusEl.title = `WebSocket: ${labels[status] || status}`;
}

function handleWsMessage(data) {
    switch (data.type) {
        case 'connected':
            console.log('WebSocket:', data.message);
            break;

        case 'alert':
            showNotification({
                title: data.title,
                message: data.message,
                severity: data.severity,
                source: data.source
            });
            break;

        case 'stats_update':
            // Update live stats if needed
            console.log('Stats update:', data);
            break;

        case 'heartbeat':
            // Silent heartbeat
            break;

        default:
            console.log('Unknown WebSocket message:', data);
    }

    // Dispatch globally for modules (like MqttManager) to listen
    window.dispatchEvent(new CustomEvent('kusanagi-ws-message', { detail: data }));
}

function showNotification(options) {
    const container = document.getElementById('notification-container');
    if (!container) return;

    const id = 'notif-' + Date.now();
    const notification = document.createElement('div');
    notification.id = id;
    notification.className = `notification ${options.severity || 'info'}`;

    notification.innerHTML = `
        <div class="notification-header">
            <span class="notification-title">${getSeverityIcon(options.severity)} ${options.title}</span>
            <button class="notification-close" onclick="dismissNotification('${id}')">&times;</button>
        </div>
        <div class="notification-body">${options.message}</div>
        ${options.source ? `<div class="notification-source">Source: ${options.source}</div>` : ''}
    `;

    container.appendChild(notification);

    // Auto-dismiss after 8 seconds
    setTimeout(() => dismissNotification(id), 8000);
}

function dismissNotification(id) {
    const notification = document.getElementById(id);
    if (notification) {
        notification.classList.add('hiding');
        setTimeout(() => notification.remove(), 300);
    }
}

function getSeverityIcon(severity) {
    const icons = {
        'error': '🔴',
        'warning': '🟠',
        'success': '🟢',
        'info': '🔵'
    };
    return icons[severity] || icons.info;
}

// === TABLE MANAGER (Sort & Search) ===
const TableManager = {
    // Store table data for filtering/sorting
    tables: {},

    // Initialize a searchable/sortable table
    init(tableId, data, renderFn, columns) {
        this.tables[tableId] = {
            data: data,
            filtered: [...data],
            renderFn: renderFn,
            columns: columns,
            sortCol: null,
            sortDir: 'asc',
            searchTerm: ''
        };
    },

    // Create search input HTML
    createSearchInput(tableId, placeholder = 'Search...') {
        return `
            <div class="table-search">
                <input type="text" 
                       class="search-input" 
                       id="search-${tableId}" 
                       placeholder="${placeholder}"
                       oninput="TableManager.search('${tableId}', this.value)">
                <span class="search-icon">🔍</span>
            </div>
        `;
    },

    // Create sortable header
    createSortableHeader(tableId, columns) {
        return columns.map((col, idx) => {
            const table = this.tables[tableId];
            const isSorted = table && table.sortCol === col.key;
            const sortIcon = isSorted ? (table.sortDir === 'asc' ? '▲' : '▼') : '⇅';
            return `<th class="sortable" onclick="TableManager.sort('${tableId}', '${col.key}')">${col.label} <span class="sort-icon">${sortIcon}</span></th>`;
        }).join('');
    },

    // Search/filter function
    search(tableId, term) {
        const table = this.tables[tableId];
        if (!table) return;

        table.searchTerm = term.toLowerCase();
        table.filtered = table.data.filter(item => {
            return Object.values(item).some(val =>
                String(val).toLowerCase().includes(table.searchTerm)
            );
        });

        table.renderFn(table.filtered);
    },

    // Sort function
    sort(tableId, column) {
        const table = this.tables[tableId];
        if (!table) return;

        // Toggle direction if same column
        if (table.sortCol === column) {
            table.sortDir = table.sortDir === 'asc' ? 'desc' : 'asc';
        } else {
            table.sortCol = column;
            table.sortDir = 'asc';
        }

        table.filtered.sort((a, b) => {
            let valA = a[column];
            let valB = b[column];

            // Handle null/undefined
            if (valA == null) valA = '';
            if (valB == null) valB = '';

            // Check if numeric
            const numA = parseFloat(valA);
            const numB = parseFloat(valB);

            if (!isNaN(numA) && !isNaN(numB)) {
                return table.sortDir === 'asc' ? numA - numB : numB - numA;
            }

            // String comparison
            const strA = String(valA).toLowerCase();
            const strB = String(valB).toLowerCase();

            if (table.sortDir === 'asc') {
                return strA.localeCompare(strB);
            } else {
                return strB.localeCompare(strA);
            }
        });

        table.renderFn(table.filtered);
    }
};

// === TAB NAVIGATION ===
async function switchTab(tabName) {
    // Deactivate previous tab dashboards
    const currentTab = window.KusanagiDashboard ? KusanagiDashboard.activeTab : null;
    if (currentTab === 'proxmox' && tabName !== 'proxmox' && window.ProxmoxDashboard) {
        if (typeof window.ProxmoxDashboard.deactivate === 'function') {
            window.ProxmoxDashboard.deactivate();
        }
    }
    if (currentTab === 'system' && tabName !== 'system' && window.KusanagiSystem) {
        if (typeof window.KusanagiSystem.deactivate === 'function') {
            window.KusanagiSystem.deactivate();
        }
    }

    // Update buttons
    document.querySelectorAll(".tab-btn").forEach(btn => {
        btn.classList.remove("active");
        if (btn.dataset.tab === tabName) {
            btn.classList.add("active");
        }
    });

    // Update content
    document.querySelectorAll(".tab-content").forEach(section => {
        if (section.dataset.tab === tabName) {
            section.style.display = "block";
            section.classList.add("active");
        } else {
            section.style.display = "none";
            section.classList.remove("active");
        }
    });

    // Show/hide dashboard header elements (only on ArgoCD page)
    const dashboardHeader = document.getElementById("dashboard-header-elements");
    if (dashboardHeader) {
        dashboardHeader.style.display = (tabName === "argocd") ? "block" : "none";
    }

    // Update active tab tracking
    if (window.KusanagiDashboard) {
        window.KusanagiDashboard.activeTab = tabName;
    }

    // Load partial if needed (for sections that support it)
    if (window.PageLoader && PageLoader.partials[tabName]) {
        const section = document.querySelector(`section[data-tab="${tabName}"]`);
        if (section && section.dataset.loaded !== 'true') {
            await PageLoader.loadPartial(tabName);
        }
    }

    // Load data for specific tabs
    if (tabName === "proxmox" && window.ProxmoxDashboard) {
        if (typeof window.ProxmoxDashboard.activate === 'function') {
            window.ProxmoxDashboard.activate();
        }
    } else if (tabName === "homeassistant" && window.HomeAssistantDashboard) {
        HomeAssistantDashboard.init();
    } else if (tabName === "weather" && window.WeatherDashboard) {
        WeatherDashboard.init();
    } else if (tabName === "security" && window.SecurityDashboard) {
        SecurityDashboard.init();
    } else if (tabName === "alerts" && window.AlertsManager) {
        AlertsManager.init();
    } else if (tabName === "system" && window.KusanagiSystem) {
        KusanagiSystem.activate();
    }
}

/**
 * Centralized refresh function for all Kusanagi data
 */
function refreshAllKusanagiData() {
    console.log("🔄 Global Kusanagi refresh triggered...");

    // Visual feedback
    const logo = document.querySelector('.header-logo');
    if (logo) {
        logo.classList.add('refreshing');
        setTimeout(() => logo.classList.remove('refreshing'), 1000);
    }

    if (typeof showNotification === 'function') {
        showNotification({
            title: "System Refresh",
            message: "Syncing all components with real-time cluster state...",
            severity: "info"
        });
    }

    // Core Kubernetes & ArgoCD Status
    if (window.K8sManager) {
        if (K8sManager.fetchArgoStatus) K8sManager.fetchArgoStatus();
        if (K8sManager.fetchNodesStatus) K8sManager.fetchNodesStatus();
        if (K8sManager.fetchClusterOverview) K8sManager.fetchClusterOverview();
        if (K8sManager.fetchEvents) K8sManager.fetchEvents(window.currentEventFilter || 'all', 1);
        if (K8sManager.fetchBackupsStatus) K8sManager.fetchBackupsStatus();
        if (K8sManager.fetchStorageStatus) K8sManager.fetchStorageStatus();
        if (K8sManager.fetchServices) K8sManager.fetchServices();
        if (K8sManager.fetchIngress) K8sManager.fetchIngress();
    }
    if (typeof fetchAppsWithResources === 'function') fetchAppsWithResources();

    // Component Managers
    if (window.MetricsManager && MetricsManager.init) MetricsManager.init();
    if (window.AlertsManager && AlertsManager.init) AlertsManager.init();
    if (window.NewsManager && NewsManager.fetchNews) NewsManager.fetchNews();
    if (window.QuotasManager && QuotasManager.fetchQuotas) {
        // QuotasManager.fetchQuotas(); // Disabled
    }
    if (window.MqttManager && MqttManager.fetchInitialData) MqttManager.fetchInitialData();

    // Refresh current active tab if it's a dashboard
    const activeTab = window.KusanagiDashboard ? KusanagiDashboard.activeTab : null;
    if (activeTab && typeof switchTab === 'function') {
        switchTab(activeTab);
    }
}

// Global dashboard state
window.KusanagiDashboard = {
    activeTab: 'argocd'
};

// Initialize WebSocket on page load
document.addEventListener('DOMContentLoaded', () => {
    initWebSocket();

    // Reconnect WebSocket when page becomes visible
    document.addEventListener("visibilitychange", () => {
        if (!document.hidden && (!wsConnection || wsConnection.readyState !== WebSocket.OPEN)) {
            wsReconnectAttempts = 0;
            initWebSocket();
        }
    });
});
