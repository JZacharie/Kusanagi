/**
 * Docker Compose to Proxmox Converter Module
 */
const ComposeConverter = {
    initialized: false,

    init() {
        if (this.initialized) return;
        this.initialized = true;
        
        // Handle full feature visibility
        const fullFeatureElements = document.querySelectorAll('.full-feature-only');
        if (window.KUSANAGI_FULL_FEATURES) {
            fullFeatureElements.forEach(el => el.style.display = 'block');
        }
        
        console.log('🔧 Compose Converter initialized');
    },

    async deploy() {
        const yaml = document.getElementById('compose-yaml').value;
        const targetNode = document.getElementById('compose-target-node').value;
        const resultsContainer = document.getElementById('compose-results');
        const resultsList = document.getElementById('compose-results-list');
        
        if (!yaml.trim()) {
            window.showNotification('Please provide a docker-compose.yml content', 'error');
            return;
        }

        try {
            window.showNotification('Deploying stack to Proxmox...', 'info');
            resultsContainer.style.display = 'block';
            resultsList.innerHTML = '<div class="loading">Deploying services...</div>';

            const response = await api.post('/api/proxmox/deploy-compose', {
                yaml: yaml,
                target_node: targetNode || 'aquabot'
            });

            this.renderResults(response.results);
            window.showNotification('Deployment completed', 'success');
        } catch (error) {
            console.error('Deployment failed:', error);
            window.showNotification(`Deployment failed: ${error.message}`, 'error');
            resultsList.innerHTML = `<div class="status-badge unhealthy" style="width: 100%; text-align: center;">Error: ${error.message}</div>`;
        }
    },

    renderResults(results) {
        const resultsList = document.getElementById('compose-results-list');
        
        if (!results || results.length === 0) {
            resultsList.innerHTML = '<div class="no-issues">No services found in compose file</div>';
            return;
        }

        resultsList.innerHTML = results.map(res => `
            <div style="background: rgba(0,0,0,0.4); border-left: 3px solid ${res.status === 'success' ? 'var(--neon-green)' : 'var(--neon-pink)'}; padding: 0.75rem 1rem; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; flex-direction: column;">
                    <span style="font-weight: bold; color: ${res.status === 'success' ? 'var(--neon-green)' : 'var(--neon-pink)'}">${res.service_name}</span>
                    <span style="font-size: 0.8rem; color: var(--text-secondary);">${res.message}</span>
                </div>
                <span class="status-badge ${res.status === 'success' ? 'healthy' : 'unhealthy'}">${res.status}</span>
            </div>
        `).join('');
    }
};

// Expose to window
window.ComposeConverter = ComposeConverter;

// Hook into page activation
document.addEventListener('DOMContentLoaded', () => {
    // Check for page switch and init if needed
    window.addEventListener('hashchange', () => {
        if (window.location.hash === '#compose') {
            ComposeConverter.init();
        }
    });

    // Handle initial load
    if (window.location.hash === '#compose') {
        ComposeConverter.init();
    }
});
