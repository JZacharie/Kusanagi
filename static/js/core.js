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

    // Batch DOM reads first
    const tabButtons = document.querySelectorAll(".tab-btn");
    const tabContents = document.querySelectorAll(".tab-content");
    const dashboardHeader = document.getElementById("dashboard-header-elements");

    // Batch DOM writes (minimize reflows)
    // Update buttons
    tabButtons.forEach(btn => {
        btn.classList.toggle("active", btn.dataset.tab === tabName);
    });

    // Update content - use CSS hidden attribute instead of display
    tabContents.forEach(section => {
        const isTarget = section.dataset.tab === tabName;
        section.classList.toggle("active", isTarget);
        section.hidden = !isTarget;
    });

    // Show/hide dashboard header elements (only on ArgoCD page)
    if (dashboardHeader) {
        dashboardHeader.hidden = (tabName !== "argocd");
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

    // Load data for specific tabs (avoiding duplicates if already handled by PageLoader/initScripts)
    if (tabName === "proxmox" && window.ProxmoxDashboard) {
        if (typeof window.ProxmoxDashboard.activate === 'function') {
            window.ProxmoxDashboard.activate();
        }
    } else if (tabName === "homeassistant" && window.HomeAssistantDashboard) {
        // init is handled by PageLoader.initScripts
    } else if (tabName === "weather" && window.WeatherDashboard) {
        // init is handled by PageLoader.initScripts
    } else if (tabName === "security" && window.SecurityDashboard) {
        // init is handled by PageLoader.initScripts
    } else if (tabName === "monitors" && window.MonitorsManager) {
        // init is handled by PageLoader.initScripts
    } else if (tabName === "system" && window.KusanagiSystem) {
        // activate is handled by PageLoader.initScripts
    } else if (tabName === "argocd" && window.K8sManager) {
        console.log("🔄 Switched to ArgoCD tab, fetching status...");
        K8sManager.fetchArgoStatus();
    } else if (tabName === "services" && window.K8sServices) {
        console.log("🔄 Switched to Services tab, fetching...");
        K8sServices.fetchServices();
    } else if (tabName === "ingress" && window.K8sServices) {
        console.log("🔄 Switched to Ingress tab, fetching...");
        K8sServices.fetchIngress();
    }

    // Emit tab change event for tab-aware modules (after DOM is ready)
    document.dispatchEvent(new CustomEvent('tabChanged', { detail: { tab: tabName } }));
}

/**
 * Centralized refresh function - now tab-aware
 * Only refreshes the currently active tab
 * @param {boolean} manualTrigger - Whether this was called by user interaction (shows notification)
 */
function refreshAllKusanagiData(manualTrigger = false) {
    const activeTab = window.KusanagiDashboard?.activeTab || 'argocd';
    console.log(`🔄 Refresh requested for active tab: ${activeTab} (manual: ${manualTrigger})`);

    // Visual feedback
    const logo = document.querySelector('.header-logo');
    if (logo) {
        logo.classList.add('refreshing');
        setTimeout(() => logo.classList.remove('refreshing'), 1000);
    }

    if (manualTrigger && typeof showNotification === 'function') {
        showNotification({
            title: "Refresh",
            message: `Refreshing ${activeTab} data...`,
            severity: "info"
        });
    }

    // Use TabManager if available (tab-aware)
    if (window.TabManager) {
        TabManager.refreshCurrentTab();
        return;
    }

    // Fallback: only refresh current tab via K8sManager
    if (window.K8sManager && K8sManager.refreshCurrentTab) {
        K8sManager.refreshCurrentTab();
    }
}

// Global dashboard state
window.KusanagiDashboard = {
    activeTab: 'argocd'
};

// Export TableManager for module access
window.TableManager = TableManager;

// Export refresh function for module access
window.refreshAllKusanagiData = refreshAllKusanagiData;

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
