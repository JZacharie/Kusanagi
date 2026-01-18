/**
 * Error Boundary Integration
 * Structured error logging and categorization for Kusanagi
 */

(function () {
    /**
     * Error Categories
     */
    const ErrorCategory = {
        NETWORK: 'network_error',
        RUNTIME: 'runtime_error',
        RESOURCE: 'resource_error',
        API: 'api_error',
        WEBSOCKET: 'websocket_error',
        UNKNOWN: 'unknown_error'
    };

    /**
     * Error Severity Levels
     */
    const ErrorSeverity = {
        CRITICAL: 'critical',
        ERROR: 'error',
        WARNING: 'warning',
        INFO: 'info'
    };

    /**
     * Categorize error based on type and context
     */
    function categorizeError(error, context = {}) {
        if (error instanceof TypeError && error.message.includes('fetch')) {
            return ErrorCategory.NETWORK;
        }
        if (error.message?.includes('WebSocket')) {
            return ErrorCategory.WEBSOCKET;
        }
        if (context.isApiError) {
            return ErrorCategory.API;
        }
        if (error.name === 'ResourceError' || error.target?.tagName) {
            return ErrorCategory.RESOURCE;
        }
        if (error instanceof Error) {
            return ErrorCategory.RUNTIME;
        }
        return ErrorCategory.UNKNOWN;
    }

    /**
     * Determine error severity
     */
    function determineErrorSeverity(error, category) {
        // Critical errors that break functionality
        if (category === ErrorCategory.WEBSOCKET ||
            error.message?.includes('fatal') ||
            error.message?.includes('critical')) {
            return ErrorSeverity.CRITICAL;
        }

        // Network errors are typically errors but not critical
        if (category === ErrorCategory.NETWORK) {
            return ErrorSeverity.ERROR;
        }

        // Resource loading failures are warnings
        if (category === ErrorCategory.RESOURCE) {
            return ErrorSeverity.WARNING;
        }

        return ErrorSeverity.ERROR;
    }

    /**
     * Enhanced error handler
     */
    window.addEventListener('error', function (event) {
        const error = event.error || {};
        const category = categorizeError(error);
        const severity = determineErrorSeverity(error, category);

        const errorContext = {
            category: category,
            severity: severity,
            message: event.message || error.message || 'Unknown error',
            filename: event.filename,
            line: event.lineno,
            column: event.colno,
            stack: error.stack,
            timestamp: new Date().toISOString(),
            url: window.location.href,
            user_agent: navigator.userAgent,
        };

        // Log to OpenObserve
        if (typeof openobserveLogs !== 'undefined') {
            const logLevel = severity === ErrorSeverity.CRITICAL ? 'error' :
                severity === ErrorSeverity.ERROR ? 'error' :
                    severity === ErrorSeverity.WARNING ? 'warn' : 'info';

            openobserveLogs.logger[logLevel]('JavaScript Error', errorContext);
        }

        // Track as RUM action
        if (typeof openobserveRum !== 'undefined') {
            openobserveRum.addAction('error_occurred', errorContext);
        }

        console.error('🔴 Kusanagi Error:', errorContext);
    }, true);

    /**
     * Enhanced promise rejection handler
     */
    window.addEventListener('unhandledrejection', function (event) {
        const error = event.reason;
        const category = categorizeError(error, { isApiError: false });
        const severity = determineErrorSeverity(error, category);

        const errorContext = {
            category: category,
            severity: severity,
            message: error?.message || String(error) || 'Unhandled promise rejection',
            stack: error?.stack,
            promise: event.promise,
            timestamp: new Date().toISOString(),
            url: window.location.href,
        };

        // Log to OpenObserve
        if (typeof openobserveLogs !== 'undefined') {
            openobserveLogs.logger.error('Unhandled Promise Rejection', errorContext);
        }

        // Track as RUM action
        if (typeof openobserveRum !== 'undefined') {
            openobserveRum.addAction('promise_rejection', errorContext);
        }

        console.error('🔴 Kusanagi Promise Rejection:', errorContext);
    });

    /**
     * Resource loading error handler
     */
    window.addEventListener('error', function (event) {
        if (event.target && (event.target.tagName === 'IMG' ||
            event.target.tagName === 'SCRIPT' ||
            event.target.tagName === 'LINK')) {

            const errorContext = {
                category: ErrorCategory.RESOURCE,
                severity: ErrorSeverity.WARNING,
                resource_type: event.target.tagName.toLowerCase(),
                resource_url: event.target.src || event.target.href,
                timestamp: new Date().toISOString(),
            };

            if (typeof openobserveLogs !== 'undefined') {
                openobserveLogs.logger.warn('Resource Loading Failed', errorContext);
            }

            if (typeof openobserveRum !== 'undefined') {
                openobserveRum.addAction('resource_error', errorContext);
            }
        }
    }, true);

    /**
     * Public API for manual error reporting
     */
    window.KusanagiErrorBoundary = {
        /**
         * Report a custom error
         */
        reportError(error, context = {}) {
            const category = categorizeError(error, context);
            const severity = context.severity || determineErrorSeverity(error, category);

            const errorContext = {
                category: category,
                severity: severity,
                message: error.message || String(error),
                stack: error.stack,
                custom_context: context,
                timestamp: new Date().toISOString(),
            };

            if (typeof openobserveLogs !== 'undefined') {
                const logLevel = severity === ErrorSeverity.CRITICAL ? 'error' :
                    severity === ErrorSeverity.ERROR ? 'error' :
                        severity === ErrorSeverity.WARNING ? 'warn' : 'info';

                openobserveLogs.logger[logLevel]('Custom Error Report', errorContext);
            }

            if (typeof openobserveRum !== 'undefined') {
                openobserveRum.addAction('custom_error', errorContext);
            }
        },

        /**
         * Wrap a function with error boundary
         */
        wrap(fn, context = {}) {
            return function (...args) {
                try {
                    const result = fn.apply(this, args);

                    // Handle promises
                    if (result && typeof result.catch === 'function') {
                        return result.catch(error => {
                            window.KusanagiErrorBoundary.reportError(error, {
                                ...context,
                                function_name: fn.name || 'anonymous',
                                arguments: args
                            });
                            throw error;
                        });
                    }

                    return result;
                } catch (error) {
                    window.KusanagiErrorBoundary.reportError(error, {
                        ...context,
                        function_name: fn.name || 'anonymous',
                        arguments: args
                    });
                    throw error;
                }
            };
        },

        /**
         * Categories and severities
         */
        Category: ErrorCategory,
        Severity: ErrorSeverity
    };

    console.log('✅ Error Boundary Integration initialized');
})();
