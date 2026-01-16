/**
 * Kusanagi RUM (Real User Monitoring) Module
 * Simplified telemetry for vanilla JavaScript
 * Sends events to OpenObserve for observability
 */

const KusanagiRUM = {
    config: {
        serviceName: 'kusanagi',
        serviceVersion: '0.6.0',
        environment: 'production',
        endpoint: 'https://openobserve.p.zacharie.org/api/default/v1/logs',
        enabled: true,
        sessionId: null,
        userId: 'anonymous'
    },

    /**
     * Initialize RUM tracking
     */
    init(options = {}) {
        Object.assign(this.config, options);
        this.config.sessionId = this.generateSessionId();

        // Track page load
        this.trackPageLoad();

        // Track navigation
        this.setupNavigationTracking();

        // Track errors
        this.setupErrorTracking();

        // Track user interactions
        this.setupInteractionTracking();

        console.log('✅ Kusanagi RUM initialized', this.config.serviceName);
    },

    /**
     * Generate unique session ID
     */
    generateSessionId() {
        return 'session_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    },

    /**
     * Send event to OpenObserve
     */
    async sendEvent(eventType, data = {}) {
        if (!this.config.enabled) return;

        const event = {
            timestamp: new Date().toISOString(),
            service: this.config.serviceName,
            version: this.config.serviceVersion,
            environment: this.config.environment,
            session_id: this.config.sessionId,
            user_id: this.config.userId,
            event_type: eventType,
            url: window.location.href,
            user_agent: navigator.userAgent,
            ...data
        };

        try {
            // For now, just log to console (OpenObserve integration requires auth)
            if (this.config.debug) {
                console.log('📊 RUM Event:', eventType, event);
            }

            // Store locally for now
            this.storeEvent(event);
        } catch (error) {
            console.error('Failed to send RUM event:', error);
        }
    },

    /**
     * Store events locally (for debugging/display)
     */
    storeEvent(event) {
        const events = JSON.parse(sessionStorage.getItem('kusanagi_rum_events') || '[]');
        events.push(event);
        // Keep last 100 events
        if (events.length > 100) events.shift();
        sessionStorage.setItem('kusanagi_rum_events', JSON.stringify(events));
    },

    /**
     * Get stored events
     */
    getEvents() {
        return JSON.parse(sessionStorage.getItem('kusanagi_rum_events') || '[]');
    },

    /**
     * Track page load performance
     */
    trackPageLoad() {
        window.addEventListener('load', () => {
            const perf = performance.timing;
            const loadTime = perf.loadEventEnd - perf.navigationStart;
            const domReady = perf.domContentLoadedEventEnd - perf.navigationStart;
            const ttfb = perf.responseStart - perf.requestStart;

            this.sendEvent('page_load', {
                load_time_ms: loadTime,
                dom_ready_ms: domReady,
                ttfb_ms: ttfb,
                page_title: document.title
            });
        });
    },

    /**
     * Track navigation changes
     */
    setupNavigationTracking() {
        // Track hash changes
        window.addEventListener('hashchange', (e) => {
            this.sendEvent('navigation', {
                from: e.oldURL,
                to: e.newURL,
                type: 'hash_change'
            });
        });

        // Track visibility changes
        document.addEventListener('visibilitychange', () => {
            this.sendEvent('visibility', {
                visible: !document.hidden,
                state: document.visibilityState
            });
        });
    },

    /**
     * Track JavaScript errors
     */
    setupErrorTracking() {
        window.addEventListener('error', (e) => {
            this.sendEvent('error', {
                message: e.message,
                filename: e.filename,
                line: e.lineno,
                column: e.colno,
                error_type: 'javascript_error'
            });
        });

        window.addEventListener('unhandledrejection', (e) => {
            this.sendEvent('error', {
                message: e.reason?.message || String(e.reason),
                error_type: 'unhandled_promise_rejection'
            });
        });
    },

    /**
     * Track user interactions (clicks on buttons, links)
     */
    setupInteractionTracking() {
        document.addEventListener('click', (e) => {
            const target = e.target.closest('button, a, .ext-link, .sync-button');
            if (target) {
                const tagName = target.tagName.toLowerCase();
                const text = target.textContent?.trim().substring(0, 50);
                const id = target.id || null;
                const className = target.className;

                this.sendEvent('click', {
                    element: tagName,
                    text: text,
                    id: id,
                    class: className,
                    href: target.href || null
                });
            }
        });
    },

    /**
     * Track custom events
     */
    track(eventName, data = {}) {
        this.sendEvent(eventName, data);
    },

    /**
     * Track API calls
     */
    trackApiCall(endpoint, duration, success, statusCode = null) {
        this.sendEvent('api_call', {
            endpoint: endpoint,
            duration_ms: duration,
            success: success,
            status_code: statusCode
        });
    },

    /**
     * Get session stats
     */
    getSessionStats() {
        const events = this.getEvents();
        const errors = events.filter(e => e.event_type === 'error').length;
        const clicks = events.filter(e => e.event_type === 'click').length;
        const apiCalls = events.filter(e => e.event_type === 'api_call').length;

        return {
            session_id: this.config.sessionId,
            total_events: events.length,
            errors: errors,
            clicks: clicks,
            api_calls: apiCalls,
            started: events[0]?.timestamp || null
        };
    }
};

// Auto-initialize on load
if (typeof window !== 'undefined') {
    window.KusanagiRUM = KusanagiRUM;
}
