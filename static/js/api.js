/**
 * Kusanagi API Fetch Helper
 * 
 * Standardizes all HTTP API calls with the consistent envelope format:
 * Success: { "success": true, "data": <payload> }
 * Error:   { "success": false, "error": "message" }
 */

/**
 * Fetch an API endpoint and unwrap the standard envelope.
 * Returns the `data` payload on success.
 * Throws an Error with `error` message on failure (HTTP or app-level).
 * 
 * @param {string} url - The API endpoint URL
 * @param {Object} options - Fetch options (method, headers, body, etc.)
 * @returns {Promise<any>} The unwrapped data payload
 * @throws {Error} On HTTP error or API failure
 */
async function apiFetch(url, options = {}) {
    const response = await fetch(url, options);
    const body = await response.json();
    
    if (!response.ok || body.success === false) {
        throw new Error(body.error || `Request failed (${response.status})`);
    }
    
    return body.data;
}

/**
 * Convenience methods for common HTTP verbs
 */
const api = {
    /**
     * GET request
     * @param {string} url 
     * @param {Object} options 
     * @returns {Promise<any>}
     */
    get(url, options = {}) {
        return apiFetch(url, { ...options, method: 'GET' });
    },

    /**
     * POST request
     * @param {string} url 
     * @param {Object} body 
     * @param {Object} options 
     * @returns {Promise<any>}
     */
    post(url, body = null, options = {}) {
        const opts = {
            ...options,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            }
        };
        if (body) {
            opts.body = JSON.stringify(body);
        }
        return apiFetch(url, opts);
    },

    /**
     * DELETE request
     * @param {string} url 
     * @param {Object} options 
     * @returns {Promise<any>}
     */
    delete(url, options = {}) {
        return apiFetch(url, { ...options, method: 'DELETE' });
    }
};

// Global exports
window.apiFetch = apiFetch;
window.api = api;
