/**
 * Kusanagi Debug Module
 * Provides detailed logging and diagnostics for API responses
 */

const KusanagiDebug = {
    enabled: true,
    logLevel: 'debug', // 'debug', 'info', 'warn', 'error'
    
    // Enable/disable debug mode
    setEnabled(enabled) {
        this.enabled = enabled;
        console.log(`[KusanagiDebug] ${enabled ? 'Enabled' : 'Disabled'}`);
    },
    
    // Log API responses with detailed info
    logApiResponse(endpoint, data, duration) {
        if (!this.enabled) return;
        
        console.group(`📡 API Response: ${endpoint}`);
        console.log(`⏱️ Duration: ${duration?.toFixed(2) || '?'}ms`);
        console.log('📦 Data:', data);
        
        // Check for common data issues
        if (data === null || data === undefined) {
            console.warn('⚠️ Response data is null/undefined');
        } else if (typeof data !== 'object') {
            console.warn(`⚠️ Response data is not an object: ${typeof data}`);
        } else if (Object.keys(data).length === 0) {
            console.warn('⚠️ Response data is empty object');
        }
        
        // Check for error field
        if (data?.error) {
            console.error('❌ API returned error:', data.error);
        }
        
        console.groupEnd();
    },
    
    // Log API errors
    logApiError(endpoint, error, duration) {
        if (!this.enabled) return;
        
        console.group(`❌ API Error: ${endpoint}`);
        console.error('Error:', error);
        console.log(`⏱️ Duration: ${duration?.toFixed(2) || '?'}ms`);
        console.log('Stack:', error.stack);
        console.groupEnd();
    },
    
    // Validate pods data structure
    validatePodsData(data) {
        if (!this.enabled) return true;
        
        console.group('🔍 Validating Pods Data');
        
        let isValid = true;
        const requiredFields = ['total_pods', 'running_pods', 'error_pods', 'pods_in_error'];
        
        for (const field of requiredFields) {
            if (!(field in data)) {
                console.error(`❌ Missing required field: ${field}`);
                isValid = false;
            }
        }
        
        if (data.pods_in_error && !Array.isArray(data.pods_in_error)) {
            console.error('❌ pods_in_error is not an array');
            isValid = false;
        }
        
        if (data.pods_in_error && Array.isArray(data.pods_in_error)) {
            console.log(`✅ Found ${data.pods_in_error.length} pods in error`);
            
            if (data.pods_in_error.length > 0) {
                console.log('📋 Sample pod structure:', data.pods_in_error[0]);
                
                const podFields = ['name', 'namespace', 'status', 'reason', 'restart_count', 'age', 'node'];
                const sample = data.pods_in_error[0];
                for (const field of podFields) {
                    if (!(field in sample)) {
                        console.warn(`⚠️ Pod missing field: ${field}`);
                    }
                }
            }
        }
        
        console.log(`📊 Stats: ${data.total_pods} total, ${data.running_pods} running, ${data.error_pods} error`);
        console.groupEnd();
        
        return isValid;
    },
    
    // Validate network data structure
    validateNetworkData(data) {
        if (!this.enabled) return true;
        
        console.group('🔍 Validating Network Data');
        
        let isValid = true;
        const requiredFields = ['flows', 'matrix', 'namespaces', 'total_flows'];
        
        for (const field of requiredFields) {
            if (!(field in data)) {
                console.error(`❌ Missing required field: ${field}`);
                isValid = false;
            }
        }
        
        if (data.flows && !Array.isArray(data.flows)) {
            console.error('❌ flows is not an array');
            isValid = false;
        }
        
        if (data.matrix && !Array.isArray(data.matrix)) {
            console.error('❌ matrix is not an array');
            isValid = false;
        }
        
        if (data.flows && Array.isArray(data.flows)) {
            console.log(`✅ Found ${data.flows.length} flows`);
            
            if (data.flows.length > 0) {
                console.log('📋 Sample flow structure:', data.flows[0]);
            }
        }
        
        console.groupEnd();
        
        return isValid;
    },
    
    // Test API endpoints
    async testEndpoint(url, options = {}) {
        console.group(`🧪 Testing: ${url}`);
        const start = performance.now();
        
        try {
            const response = await fetch(url, options);
            const duration = performance.now() - start;
            
            console.log(`✅ Status: ${response.status} ${response.statusText}`);
            console.log(`⏱️ Time: ${duration.toFixed(2)}ms`);
            
            const contentType = response.headers.get('content-type');
            console.log(`📄 Content-Type: ${contentType}`);
            
            if (response.ok && contentType?.includes('application/json')) {
                const data = await response.json();
                console.log('📦 Response:', data);
                return { success: true, data, duration };
            } else {
                const text = await response.text();
                console.log('📄 Response (text):', text.substring(0, 500));
                return { success: false, text, duration };
            }
        } catch (error) {
            const duration = performance.now() - start;
            console.error('❌ Error:', error);
            return { success: false, error: error.message, duration };
        } finally {
            console.groupEnd();
        }
    },
    
    // Run all diagnostics
    async runDiagnostics() {
        console.log('╔════════════════════════════════════════════════════════╗');
        console.log('║     KUSANAGI DEBUG DIAGNOSTICS                         ║');
        console.log('╚════════════════════════════════════════════════════════╝');
        
        // Test Pods API
        await this.testEndpoint('/api/pods/status');
        
        // Test Network APIs
        await this.testEndpoint('/api/cilium/namespaces');
        await this.testEndpoint('/api/cilium/flows');
        await this.testEndpoint('/api/cilium/matrix');
        
        // Test other APIs
        await this.testEndpoint('/api/argocd/status');
        await this.testEndpoint('/api/nodes/status');
        
        console.log('╔════════════════════════════════════════════════════════╗');
        console.log('║     DIAGNOSTICS COMPLETE                               ║');
        console.log('╚════════════════════════════════════════════════════════╝');
    }
};

// Expose to window for console access
window.KusanagiDebug = KusanagiDebug;

// Auto-enable in development
if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
    KusanagiDebug.setEnabled(true);
    console.log('💡 KusanagiDebug: Type KusanagiDebug.runDiagnostics() to test all APIs');
}
