const KusanagiSetup = {
    envVariables: [],
    features: [],
    currentStep: 0,

    init: async function () {
        console.log('🛠️ Registering Setup module...');
        await this.fetchStatus();
    },

    fetchStatus: async function () {
        try {
            const response = await fetch('/api/setup/status');
            const data = await response.json();
            this.envVariables = data.env_variables;
            this.features = data.features;
            this.renderDashboard();
        } catch (error) {
            console.error('Failed to fetch setup status:', error);
            KusanagiNetwork.showNotification('Error', 'Failed to fetch setup status', 'error');
        }
    },

    renderDashboard: function () {
        const container = document.getElementById('setup-dashboard');
        if (!container) return;

        let html = `
            <div class="setup-grid">
                <div class="features-status-card">
                    <h3 class="card-subtitle">Feature Activation</h3>
                    <div class="feature-list">
                        ${this.features.map(f => `
                            <div class="feature-item ${f.active ? 'active' : 'inactive'}">
                                <span class="feature-name">${f.name}</span>
                                <span class="feature-badge">${f.active ? 'ENABLED' : 'DISABLED'}</span>
                                ${!f.active ? `<div class="missing-vars">Missing: ${f.missing_vars.join(', ')}</div>` : ''}
                            </div>
                        `).join('')}
                    </div>
                </div>
                
                <div class="wizard-card">
                    <h3 class="card-subtitle">Configuration Wizard</h3>
                    <div id="wizard-steps-container">
                        ${this.renderWizardStep()}
                    </div>
                </div>
            </div>
        `;
        container.innerHTML = html;
    },

    renderWizardStep: function () {
        if (this.currentStep >= this.envVariables.length) {
            return `
                <div class="wizard-completed">
                    <p>All core variables reviewed!</p>
                    <button class="cyber-btn" onclick="KusanagiSetup.restartWizard()">Restart Interview</button>
                    <button class="cyber-btn" onclick="location.reload()" style="border-color: var(--neon-cyan);">Apply & Refresh</button>
                </div>
            `;
        }

        const variable = this.envVariables[this.currentStep];
        return `
            <div class="wizard-step">
                <div class="step-header">
                    <span class="step-count">Step ${this.currentStep + 1} of ${this.envVariables.length}</span>
                    <h4>${variable.key}</h4>
                </div>
                <p class="step-desc">${variable.description}</p>
                <div class="input-group">
                    <input type="${variable.is_secret ? 'password' : 'text'}" 
                           id="setup-input-${variable.key}" 
                           class="chat-input" 
                           placeholder="Example: ${variable.example}"
                           oninput="KusanagiSetup.validateInput('${variable.key}', this.value)">
                </div>
                <div id="validation-msg-${variable.key}" class="validation-msg"></div>
                
                <div class="wizard-footer">
                    <button class="cyber-btn" onclick="KusanagiSetup.prevStep()" ${this.currentStep === 0 ? 'disabled' : ''}>Previous</button>
                    <button class="cyber-btn" onclick="KusanagiSetup.nextStep()" style="border-color: var(--neon-magenta);">Next Step</button>
                </div>
            </div>
        `;
    },

    validateInput: async function (key, value) {
        if (!value) return;

        try {
            const response = await fetch('/api/setup/validate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ key, value })
            });
            const result = await response.json();

            const msgEl = document.getElementById(`validation-msg-${key}`);
            if (msgEl) {
                msgEl.textContent = result.message;
                msgEl.className = `validation-msg ${result.valid ? 'valid' : 'invalid'}`;
            }
        } catch (error) {
            console.error('Validation error:', error);
        }
    },

    nextStep: function () {
        this.currentStep++;
        this.renderDashboard();
    },

    prevStep: function () {
        if (this.currentStep > 0) {
            this.currentStep--;
            this.renderDashboard();
        }
    },

    restartWizard: function () {
        this.currentStep = 0;
        this.renderDashboard();
    }
};

// Auto-init for switching tabs
window.addEventListener('DOMContentLoaded', () => {
    // If the hash is #setup, we could trigger it, but main dashboard handle tabs
});
