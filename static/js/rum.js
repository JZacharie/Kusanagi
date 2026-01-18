/**
 * Kusanagi RUM (Real User Monitoring) Module
 * OpenObserve Browser SDK Integration
 * https://openobserve.ai/docs/user-guide/rum/
 */

// Import OpenObserve RUM and Logs SDKs via CDN
// These scripts should be loaded in the HTML before this file

(function () {
    // Configuration
    const options = {
        clientToken: 'rumh1ycW8AuNCvCSMr4',
        applicationId: 'kusanagi-dashboard',
        site: 'o2-openobserve.p.zacharie.org',
        service: 'kusanagi',
        env: window.location.hostname === 'localhost' ? 'development' : 'production',
        version: '0.7.0',
        organizationIdentifier: 'default',
        insecureHTTP: false,
        apiVersion: 'v1',
    };

    // Wait for OpenObserve SDK to be loaded
    function initializeRUM() {
        if (typeof openobserveRum === 'undefined' || typeof openobserveLogs === 'undefined') {
            const attempt = (initializeRUM.attempts || 0) + 1;
            initializeRUM.attempts = attempt;

            // Log warning only every 5 attempts to reduce spam
            if (attempt % 5 === 0) {
                console.warn(`⏳ OpenObserve RUM SDK not loaded yet (Attempt ${attempt}), retrying...`);
            }

            // Give up after 30 seconds (approx 60 attempts)
            if (attempt > 60) {
                console.error('❌ Failed to load OpenObserve RUM SDK after 30 seconds');
                return;
            }

            setTimeout(initializeRUM, 500);
            return;
        }

        try {
            // Initialize RUM SDK
            openobserveRum.init({
                applicationId: options.applicationId,
                clientToken: options.clientToken,
                site: options.site,
                organizationIdentifier: options.organizationIdentifier,
                service: options.service,
                env: options.env,
                version: options.version,
                trackResources: true,
                trackLongTasks: true,
                trackUserInteractions: true,
                apiVersion: options.apiVersion,
                insecureHTTP: options.insecureHTTP,
                defaultPrivacyLevel: 'allow', // 'allow', 'mask-user-input', or 'mask'
                sessionSampleRate: 100, // Track 100% of sessions
                sessionReplaySampleRate: 100, // Record 100% of sessions
            });

            // Initialize Logs SDK
            openobserveLogs.init({
                clientToken: options.clientToken,
                site: options.site,
                organizationIdentifier: options.organizationIdentifier,
                service: options.service,
                env: options.env,
                version: options.version,
                forwardErrorsToLogs: true,
                insecureHTTP: options.insecureHTTP,
                apiVersion: options.apiVersion,
            });

            // Set user context (if authenticated user info is available)
            // For now, we'll track anonymous users with basic browser fingerprinting
            const userFingerprint = generateUserFingerprint();
            openobserveRum.setUser({
                id: userFingerprint,
                name: 'Kusanagi User',
                email: null, // Set if user auth is implemented
            });

            // Add global context attributes
            openobserveRum.setGlobalContextProperty('dashboard_type', 'kubernetes');
            openobserveRum.setGlobalContextProperty('platform', 'web');
            openobserveRum.setGlobalContextProperty('user_agent', navigator.userAgent);
            openobserveRum.setGlobalContextProperty('screen_resolution', `${screen.width}x${screen.height}`);
            openobserveRum.setGlobalContextProperty('timezone', Intl.DateTimeFormat().resolvedOptions().timeZone);

            // Start session replay recording
            openobserveRum.startSessionReplayRecording();

            console.log('✅ OpenObserve RUM initialized for Kusanagi', {
                service: options.service,
                env: options.env,
                version: options.version,
                applicationId: options.applicationId,
            });

            // Log successful initialization
            openobserveLogs.logger.info('Kusanagi dashboard loaded', {
                url: window.location.href,
                timestamp: new Date().toISOString(),
            });

        } catch (error) {
            console.error('❌ Failed to initialize OpenObserve RUM:', error);
        }
    }

    /**
     * Generate a simple user fingerprint for anonymous tracking
     */
    function generateUserFingerprint() {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        ctx.textBaseline = 'top';
        ctx.font = '14px Arial';
        ctx.fillText('fingerprint', 2, 2);

        const dataURL = canvas.toDataURL();
        let hash = 0;
        for (let i = 0; i < dataURL.length; i++) {
            const char = dataURL.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash;
        }

        return `anon_${Math.abs(hash)}_${navigator.language}_${screen.width}x${screen.height}`;
    }

    /**
     * Track custom Kusanagi-specific events
     */
    window.KusanagiRUM = {
        /**
         * Track tab navigation
         */
        trackTabSwitch(tabName) {
            if (typeof openobserveRum !== 'undefined') {
                openobserveRum.addAction('tab_switch', {
                    tab_name: tabName,
                    timestamp: new Date().toISOString(),
                });
            }
        },

        /**
         * Track ArgoCD sync actions
         */
        trackArgoSync(appName, success) {
            if (typeof openobserveRum !== 'undefined') {
                openobserveRum.addAction('argocd_sync', {
                    application: appName,
                    success: success,
                    timestamp: new Date().toISOString(),
                });
            }
        },

        /**
         * Track API call performance
         */
        trackApiCall(endpoint, duration, success, statusCode) {
            if (typeof openobserveRum !== 'undefined') {
                openobserveRum.addAction('api_call', {
                    endpoint: endpoint,
                    duration_ms: duration,
                    success: success,
                    status_code: statusCode,
                    timestamp: new Date().toISOString(),
                });
            }
        },

        /**
         * Track data export actions
         */
        trackExport(format, dataType) {
            if (typeof openobserveRum !== 'undefined') {
                openobserveRum.addAction('data_export', {
                    format: format,
                    data_type: dataType,
                    timestamp: new Date().toISOString(),
                });
            }
        },

        /**
         * Track chat interactions
         */
        trackChatMessage(messageType, success) {
            if (typeof openobserveRum !== 'undefined') {
                openobserveRum.addAction('chat_message', {
                    message_type: messageType,
                    success: success,
                    timestamp: new Date().toISOString(),
                });
            }
        },

        /**
         * Log custom messages
         */
        log: {
            info(message, context = {}) {
                if (typeof openobserveLogs !== 'undefined') {
                    openobserveLogs.logger.info(message, context);
                }
            },
            warn(message, context = {}) {
                if (typeof openobserveLogs !== 'undefined') {
                    openobserveLogs.logger.warn(message, context);
                }
            },
            error(message, context = {}) {
                if (typeof openobserveLogs !== 'undefined') {
                    openobserveLogs.logger.error(message, context);
                }
            },
        },

        /**
         * User Authentication Context Management
         * Call this when user logs in/out or user info changes
         */
        setUserContext(userData) {
            if (typeof openobserveRum === 'undefined') {
                console.warn('OpenObserve RUM not initialized');
                return;
            }

            const userContext = {
                id: userData.id || userData.userId || 'anonymous',
                name: userData.name || userData.username || null,
                email: userData.email || null,
            };

            // Set user in RUM
            openobserveRum.setUser(userContext);

            // Add additional user properties if available
            if (userData.role) {
                openobserveRum.setUserProperty('role', userData.role);
            }
            if (userData.organization) {
                openobserveRum.setUserProperty('organization', userData.organization);
            }
            if (userData.permissions) {
                openobserveRum.setUserProperty('permissions', JSON.stringify(userData.permissions));
            }

            // Log user session start
            if (typeof openobserveLogs !== 'undefined') {
                openobserveLogs.logger.info('User authenticated', {
                    user_id: userContext.id,
                    user_name: userContext.name,
                    timestamp: new Date().toISOString(),
                });
            }

            console.log('✅ User context updated:', userContext);
        },

        /**
         * Manual initialization (for backward compatibility or manual control)
         */
        init: function () {
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', initializeRUM);
            } else {
                initializeRUM();
            }
        },

        /**
         * Get current session statistics
         */
        getSessionStats: function () {
            return {
                sessionId: typeof openobserveRum !== 'undefined' ? openobserveRum.getSessionId() : null,
                userAgent: navigator.userAgent,
                screenResolution: `${screen.width}x${screen.height}`,
                timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
                timestamp: new Date().toISOString()
            };
        },

        /**
         * Clear user context on logout
         */
        clearUserContext() {
            if (typeof openobserveRum === 'undefined') {
                return;
            }

            // Generate new anonymous fingerprint
            const userFingerprint = generateUserFingerprint();
            openobserveRum.setUser({
                id: userFingerprint,
                name: 'Kusanagi User',
                email: null,
            });

            if (typeof openobserveLogs !== 'undefined') {
                openobserveLogs.logger.info('User logged out');
            }

            console.log('✅ User context cleared');
        },
    };

    // Initialize RUM when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initializeRUM);
    } else {
        initializeRUM();
    }

    // Track page visibility changes
    document.addEventListener('visibilitychange', () => {
        if (typeof openobserveRum !== 'undefined') {
            openobserveRum.addAction('visibility_change', {
                visible: !document.hidden,
                state: document.visibilityState,
            });
        }
    });

})();
