/**
 * API Performance Tracker
 * Automatically intercepts fetch and XMLHttpRequest to track API performance
 */

(function () {
    // Store original fetch and XHR
    const originalFetch = window.fetch;
    const originalXHROpen = XMLHttpRequest.prototype.open;
    const originalXHRSend = XMLHttpRequest.prototype.send;

    /**
     * Intercept fetch API calls
     */
    window.fetch = async function (...args) {
        const startTime = performance.now();
        const url = typeof args[0] === 'string' ? args[0] : args[0]?.url || 'unknown';
        const method = args[1]?.method || 'GET';

        let response;
        let error = null;
        let statusCode = null;

        try {
            response = await originalFetch.apply(this, args);
            statusCode = response.status;

            // Track successful API call
            const duration = performance.now() - startTime;
            trackApiCall(url, method, duration, response.ok, statusCode);

            return response;
        } catch (err) {
            error = err;
            const duration = performance.now() - startTime;

            // Track failed API call
            trackApiCall(url, method, duration, false, null, err.message);

            throw err;
        }
    };

    /**
     * Intercept XMLHttpRequest
     */
    XMLHttpRequest.prototype.open = function (method, url, ...args) {
        this._apiTracking = {
            method: method,
            url: url,
            startTime: null
        };
        return originalXHROpen.apply(this, [method, url, ...args]);
    };

    XMLHttpRequest.prototype.send = function (...args) {
        if (this._apiTracking) {
            this._apiTracking.startTime = performance.now();

            this.addEventListener('loadend', function () {
                const tracking = this._apiTracking;
                if (!tracking) return;

                const duration = performance.now() - tracking.startTime;
                const success = this.status >= 200 && this.status < 400;

                trackApiCall(
                    tracking.url,
                    tracking.method,
                    duration,
                    success,
                    this.status,
                    this.statusText
                );
            });
        }

        return originalXHRSend.apply(this, args);
    };

    /**
     * Track API call with OpenObserve RUM
     */
    function trackApiCall(url, method, duration, success, statusCode, errorMessage = null) {
        // Skip tracking for OpenObserve endpoints to avoid recursive loops
        if (url.includes('openobserve') || url.includes('o2-')) {
            return;
        }

        if (window.KusanagiRUM) {
            window.KusanagiRUM.trackApiCall(url, duration, success, statusCode);
        }

        // Also use native RUM action tracking
        if (typeof OO_RUM !== 'undefined') {
            OO_RUM.addAction('api_call', {
                endpoint: url,
                method: method,
                duration_ms: Math.round(duration),
                success: success,
                status_code: statusCode,
                error_message: errorMessage,
                timestamp: new Date().toISOString(),
            });

            // Log errors to OpenObserve Logs
            if (!success && typeof OO_LOGS !== 'undefined') {
                OO_LOGS.logger.error('API call failed', {
                    endpoint: url,
                    method: method,
                    status_code: statusCode,
                    error_message: errorMessage,
                    duration_ms: Math.round(duration),
                });
            }
        }
    }

    console.log('✅ API Performance Tracking initialized');
})();
