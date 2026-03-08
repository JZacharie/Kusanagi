/**
 * Security Dashboard Module - Enhanced Trivy Integration
 * Features: Charts, filtering, CSV export, detailed views
 */
const SecurityDashboard = {
    data: null,
    filteredData: null,
    chart: null,
    namespaces: new Set(),

    init() {
        console.log('✅ Security Dashboard initialized');
        this.initChart();
        this.setDefaultValues();
    },

    setDefaultValues() {
        // Set default values for all stat elements
        Object.entries({
            'security-critical-vulns': '0',
            'security-high-vulns': '0',
            'security-medium-vulns': '0',
            'security-low-vulns': '0',
            'security-total-vulns': '0'
        }).forEach(([id, value]) => {
            const el = document.getElementById(id);
            if (el) el.textContent = value;
        });
    },

    initChart() {
        const canvas = document.getElementById('vuln-chart');
        if (!canvas) return;

        // Simple bar chart using canvas
        this.chart = {
            canvas: canvas,
            draw: (data) => {
                const ctx = canvas.getContext('2d');
                const width = canvas.width = 280;
                const height = canvas.height = 180;

                // Clear
                ctx.fillStyle = '#0a0a0f';
                ctx.fillRect(0, 0, width, height);

                if (!data || data.total === 0) {
                    ctx.fillStyle = '#666';
                    ctx.font = '14px JetBrains Mono';
                    ctx.textAlign = 'center';
                    ctx.fillText('No data available', width / 2, height / 2);
                    return;
                }

                const max = Math.max(data.critical, data.high, data.medium, data.low, 1);
                const barWidth = 50;
                const gap = 20;
                const startX = 20;
                const startY = height - 40;

                const bars = [
                    { label: 'Crit', value: data.critical, color: '#ff4444' },
                    { label: 'High', value: data.high, color: '#ff8800' },
                    { label: 'Med', value: data.medium, color: '#ffdd00' },
                    { label: 'Low', value: data.low, color: '#44ff44' }
                ];

                bars.forEach((bar, i) => {
                    const x = startX + i * (barWidth + gap);
                    const barHeight = (bar.value / max) * 100;

                    // Draw bar
                    ctx.fillStyle = bar.color;
                    ctx.fillRect(x, startY - barHeight, barWidth, barHeight);

                    // Draw value on top
                    ctx.fillStyle = '#fff';
                    ctx.font = '12px JetBrains Mono';
                    ctx.textAlign = 'center';
                    ctx.fillText(bar.value.toString(), x + barWidth / 2, startY - barHeight - 5);

                    // Draw label
                    ctx.fillStyle = '#888';
                    ctx.fillText(bar.label, x + barWidth / 2, startY + 15);
                });
            }
        };
    },

    loadSecurityData() {
        return this.fetchAndRender();
    },

    async fetchAndRender() {
        try {
            // Show loading
            this.renderLoading();

            const promises = [
                api.get('/api/security/vulnerabilities').catch(e => ({ error: e, type: 'vulns' })),
                api.get('/api/security/reports').catch(e => ({ error: e, type: 'reports' }))
            ];

            const results = await Promise.allSettled(promises);

            // Process Vulnerabilities
            const vulnsResult = results[0];
            if (vulnsResult.status === 'fulfilled' && vulnsResult.value && !vulnsResult.value.error) {
                const vulns = vulnsResult.value;
                this.data = vulns;
                this.filteredData = { ...vulns };

                // Extract namespaces for filter
                this.extractNamespaces(vulns.images || []);

                // Update UI
                this.renderStats(vulns);
                this.renderChart(vulns);
                this.renderVulnerabilities(vulns);
                this.renderByNamespace(vulns);
                this.renderTopRisk(vulns);
            } else {
                console.error('Failed to fetch vulnerabilities:', vulnsResult.value?.error || vulnsResult.reason);
                this.renderSecurityError('Security service unavailable. Please check configuration.');
            }

            // Process Reports
            const reportsResult = results[1];
            if (reportsResult.status === 'fulfilled' && reportsResult.value && !reportsResult.value.error) {
                this.renderReportSelector(reportsResult.value || []);
            } else {
                console.warn('Failed to fetch reports list:', reportsResult.value?.error || reportsResult.reason);
            }

            this.updateLastUpdated();

        } catch (error) {
            console.error('Critical failure in security dashboard:', error);
            this.renderSecurityError('Dashboard initialization failed.');
        }
    },

    refreshAll() {
        this.fetchAndRender();
    },

    renderLoading() {
        const containers = [
            'security-vulns-content',
            'security-by-namespace',
            'security-top-risk',
            'cilium-metrics-content'
        ];
        containers.forEach(id => {
            const el = document.getElementById(id);
            if (el) el.innerHTML = '<div class="loading">Loading...</div>';
        });
    },

    updateLastUpdated() {
        const el = document.getElementById('last-updated');
        if (el) {
            el.textContent = `Last updated: ${new Date().toLocaleTimeString()}`;
        }
    },

    extractNamespaces(images) {
        this.namespaces.clear();
        images.forEach(img => {
            if (img.namespace) this.namespaces.add(img.namespace);
        });

        // Populate namespace filter
        const select = document.getElementById('filter-namespace');
        if (select) {
            const currentValue = select.value;
            select.innerHTML = '<option value="all">All Namespaces</option>';
            Array.from(this.namespaces).sort().forEach(ns => {
                select.innerHTML += `<option value="${ns}">${ns}</option>`;
            });
            select.value = currentValue || 'all';
        }
    },

    applyFilters() {
        if (!this.data) return;

        const severityFilter = document.getElementById('filter-severity')?.value || 'all';
        const namespaceFilter = document.getElementById('filter-namespace')?.value || 'all';
        const searchFilter = document.getElementById('filter-search')?.value?.toLowerCase() || '';

        let filtered = [...(this.data.images || [])];

        // Filter by namespace
        if (namespaceFilter !== 'all') {
            filtered = filtered.filter(img => img.namespace === namespaceFilter);
        }

        // Filter by search
        if (searchFilter) {
            filtered = filtered.filter(img =>
                img.image?.toLowerCase().includes(searchFilter) ||
                img.namespace?.toLowerCase().includes(searchFilter)
            );
        }

        // Filter by severity (show only images with at least that severity)
        if (severityFilter !== 'all') {
            filtered = filtered.filter(img => {
                switch (severityFilter) {
                    case 'critical': return img.critical_count > 0;
                    case 'high': return img.critical_count > 0 || img.high_count > 0;
                    case 'medium': return img.critical_count > 0 || img.high_count > 0 || img.medium_count > 0;
                    default: return true;
                }
            });
        }

        this.filteredData = {
            ...this.data,
            images: filtered
        };

        this.renderVulnerabilities(this.filteredData);

        // Update filtered count
        const countEl = document.getElementById('filtered-count');
        if (countEl && this.data.images) {
            countEl.textContent = `Showing ${filtered.length} of ${this.data.images.length}`;
        }
    },

    renderStats(vulns) {
        // Ensure vulns is an object with default values
        const data = vulns || {};
        const stats = {
            critical: data.critical ?? 0,
            high: data.high ?? 0,
            medium: data.medium ?? 0,
            low: data.low ?? 0,
            total: data.total ?? 0
        };

        console.log('Rendering stats:', stats);

        Object.entries(stats).forEach(([key, value]) => {
            const el = document.getElementById(`security-${key}-vulns`);
            if (el) {
                el.textContent = value;
                // Add animation for changes
                el.style.transform = 'scale(1.2)';
                setTimeout(() => el.style.transform = 'scale(1)', 200);
            } else {
                console.warn(`Element security-${key}-vulns not found`);
            }
        });

        // Update document title with critical count
        if (stats.critical > 0) {
            document.title = `(${stats.critical} Critical) Security - Kusanagi`;
        } else {
            document.title = 'Security - Kusanagi';
        }
    },

    renderChart(vulns) {
        if (this.chart) {
            this.chart.draw(vulns);
        }
    },

    renderSecurityError(message) {
        // Reset stats
        ['critical', 'high', 'medium', 'low', 'total'].forEach(key => {
            const el = document.getElementById(`security-${key}-vulns`);
            if (el) el.textContent = '-';
        });

        // Render error
        const container = document.getElementById('security-vulns-content');
        if (container) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 3rem; text-align: center;">
                    <span style="font-size: 3rem;">🛡️</span>
                    <h3 style="margin: 1rem 0; color: var(--neon-cyan);">Security Center</h3>
                    <p style="color: var(--neon-orange); margin-bottom: 1rem;">⚠️ ${message}</p>
                    <button onclick="SecurityDashboard.refreshAll()" class="cyber-btn" style="margin-top: 1rem;">
                        🔄 Retry
                    </button>
                </div>
            `;
        }

        if (this.chart) this.chart.draw({ total: 0 });
    },

    renderReportSelector(reports) {
        const container = document.getElementById('security-report-selector');
        if (!container) return;

        // Store all reports for filtering
        if (reports) {
            this.allReports = reports;
        }

        const safeReports = this.allReports || [];

        // Setup container with filters if not already set up or if it's empty
        if (!document.getElementById('report-filters') || container.innerHTML === '') {
            container.innerHTML = `
                <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--neon-cyan); border-radius: 8px; padding: 0.75rem;">
                    <div id="report-filters" style="display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; margin-bottom: 0.75rem; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 0.5rem;">
                        <span style="color: var(--neon-cyan); font-size: 0.9rem; font-weight: bold; margin-right: auto; white-space: nowrap;">📊 Reports <span id="report-count">(${safeReports.length})</span></span>
                        
                        <!-- Filters -->
                        <select id="report-filter-type" onchange="SecurityDashboard.filterReports()" class="cyber-input" style="padding: 0.25rem; width: 100px; font-size: 0.8rem; background: rgba(0,0,0,0.5); color: #fff; border: 1px solid #444; border-radius: 4px;">
                            <option value="all">Type</option>
                        </select>
                        <input type="text" id="report-filter-namespace" oninput="SecurityDashboard.filterReports()" placeholder="Namespace..." class="cyber-input" style="padding: 0.25rem 0.5rem; width: 110px; font-size: 0.8rem; background: rgba(0,0,0,0.5); color: #fff; border: 1px solid #444; border-radius: 4px;">
                        <input type="text" id="report-filter-name" oninput="SecurityDashboard.filterReports()" placeholder="Search name..." class="cyber-input" style="padding: 0.25rem 0.5rem; width: 140px; font-size: 0.8rem; background: rgba(0,0,0,0.5); color: #fff; border: 1px solid #444; border-radius: 4px;">
                    </div>
                    <div id="report-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 0.5rem; max-height: 250px; overflow-y: auto;">
                        <!-- Grid content -->
                        <div class="loading">Loading reports...</div>
                    </div>
                </div>
            `;

            // Populate Types dynamically
            const types = new Set();
            safeReports.forEach(r => {
                const category = (typeof r === 'string' ? r.split('/')[0] : (r.category || 'general'));
                if (category) types.add(category);
            });
            const typeSelect = document.getElementById('report-filter-type');
            if (typeSelect && types.size > 0) {
                Array.from(types).sort().forEach(t => {
                    typeSelect.innerHTML += `<option value="${t}">${t}</option>`;
                });
            }
        }

        this.filterReports();
    },

    filterReports() {
        const typeFilter = document.getElementById('report-filter-type')?.value || 'all';
        const nsFilter = document.getElementById('report-filter-namespace')?.value?.toLowerCase() || '';
        const nameFilter = document.getElementById('report-filter-name')?.value?.toLowerCase() || '';

        const filtered = (this.allReports || []).filter(r => {
            const isString = typeof r === 'string';
            const category = isString ? r.split('/')[0] : (r.category || 'general');
            const name = isString ? r.split('/').pop() : (r.name || r.report_id);
            // Namespace heuristic: search in name for the namespace string since we don't have explicit field in list
            const searchCtx = name.toLowerCase();

            if (typeFilter !== 'all' && category !== typeFilter) return false;
            if (nsFilter && !searchCtx.includes(nsFilter)) return false;
            if (nameFilter && !searchCtx.includes(nameFilter)) return false;

            return true;
        });

        this.renderReportGrid(filtered);
    },

    renderReportGrid(reports) {
        const container = document.getElementById('report-grid');
        const countEl = document.getElementById('report-count');
        if (!container) return;

        if (countEl) countEl.textContent = `(${reports.length})`;

        if (reports.length === 0) {
            container.innerHTML = '<div style="grid-column: 1/-1; text-align: center; color: #666; padding: 1rem;">No reports match filters</div>';
            return;
        }

        const isStringArray = reports.length > 0 && typeof reports[0] === 'string';

        container.innerHTML = reports.map((r, index) => {
            const reportId = isStringArray ? r : (r.report_id || r.name);
            const reportName = isStringArray ? r.split('/').pop() : (r.name || r.report_id);
            const category = isStringArray ? r.split('/')[0] : (r.category || 'general');
            const date = !isStringArray && r.timestamp ? new Date(r.timestamp).toLocaleString() : '';

            return `
                <div onclick="SecurityDashboard.viewReportDetail('${reportId}')" 
                     class="report-card" 
                     style="cursor: pointer; padding: 0.5rem; background: rgba(0,255,255,0.05); border: 1px solid rgba(0,255,255,0.2); border-radius: 4px; transition: all 0.2s; display: flex; align-items: center; gap: 0.5rem;"
                     onmouseover="this.style.background='rgba(0,255,255,0.1)'; this.style.borderColor='var(--neon-cyan)'" 
                     onmouseout="this.style.background='rgba(0,255,255,0.05)'; this.style.borderColor='rgba(0,255,255,0.2)'">
                    <span style="font-size: 1.2rem;">📄</span>
                    <div style="flex: 1; min-width: 0;">
                        <div style="font-size: 0.85rem; color: var(--neon-cyan); font-weight: bold; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${reportName}">${reportName}</div>
                        <div style="font-size: 0.7rem; color: #888; display: flex; justify-content: space-between;">
                            <span>${category}</span>
                            <span>${date}</span>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    },

    async viewReportDetail(reportId) {
        const modal = document.getElementById('report-detail-modal');
        const content = document.getElementById('report-detail-content');

        if (!modal || !content) return;

        modal.style.display = 'flex';
        content.innerHTML = '<div class="loading">Loading report details...</div>';

        try {
            // Parse category/name from reportId
            const parts = reportId.split('/');
            let url;
            if (parts.length >= 2) {
                const category = parts[0];
                const name = parts.slice(1).join('/');
                url = `/api/security/reports/${encodeURIComponent(category)}/${encodeURIComponent(name)}`;
            } else {
                // Si c'est juste un ID simple
                url = `/api/security/reports/general/${encodeURIComponent(reportId)}`;
            }

            const report = await api.get(url);
            this.renderReportDetail(report, reportId);
        } catch (error) {
            console.error('Failed to load report:', error);
            content.innerHTML = `
                <div style="padding: 2rem; text-align: center;">
                    <span style="font-size: 3rem;">❌</span>
                    <h3 style="color: #ff4444; margin: 1rem 0;">Failed to load report</h3>
                    <p style="color: #888;">${error.message || 'Unknown error'}</p>
                    <button onclick="SecurityDashboard.viewReportDetail('${reportId}')" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                </div>
            `;
        }
    },

    renderReportDetail(report, reportId) {
        const content = document.getElementById('report-detail-content');
        if (!content) return;

        const enrichment = report.enrichment || {};
        const originalData = report.original_data || {};
        // Real Trivy JSON: original_data.report.vulnerabilities
        // Legacy format:   original_data.vulnerabilities
        const vulnerabilities = (originalData.report?.vulnerabilities)
            || (originalData.Report?.Vulnerabilities)
            || originalData.vulnerabilities
            || [];
        const metadata = originalData.metadata || {};
        // Use Trivy summary block if available
        const trivySummary = originalData.report?.summary || null;

        // Count severities
        const severityCount = { CRITICAL: 0, HIGH: 0, MEDIUM: 0, LOW: 0, UNKNOWN: 0 };
        if (trivySummary) {
            // Fast path: use Trivy's pre-computed summary
            severityCount.CRITICAL = trivySummary.criticalCount || 0;
            severityCount.HIGH = trivySummary.highCount || 0;
            severityCount.MEDIUM = trivySummary.mediumCount || 0;
            severityCount.LOW = trivySummary.lowCount || 0;
        } else {
            vulnerabilities.forEach(v => {
                const sev = (v.severity || 'UNKNOWN').toUpperCase();
                if (severityCount[sev] !== undefined) severityCount[sev]++;
                else severityCount.UNKNOWN++;
            });
        }

        content.innerHTML = `
            <div style="padding: 1rem;">
                <!-- Header -->
                <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 1.5rem; padding-bottom: 1rem; border-bottom: 1px solid rgba(255,255,255,0.1);">
                    <div>
                        <h2 style="margin: 0 0 0.5rem 0; color: var(--neon-cyan);">${report.name || reportId}</h2>
                        <div style="color: #888; font-size: 0.9rem;">
                            <span style="margin-right: 1rem;">📁 ${report.report_type || 'Security Report'}</span>
                            <span>🕐 ${report.timestamp ? new Date(report.timestamp).toLocaleString() : 'N/A'}</span>
                        </div>
                    </div>
                    ${report.enrichment ? `
                        <div style="text-align: right;">
                            <div style="font-size: 0.8rem; color: #888;">AI Criticality Score</div>
                            <div style="font-size: 2rem; font-weight: bold; color: ${this.getScoreColor(enrichment.criticality_score || 0)};">
                                ${(enrichment.criticality_score || 0).toFixed(1)}
                            </div>
                        </div>
                    ` : ''}
                </div>

                <!-- AI Enrichment -->
                ${report.enrichment ? `
                    <div style="background: rgba(255,0,128,0.05); border: 1px solid rgba(255,0,128,0.3); border-radius: 8px; padding: 1rem; margin-bottom: 1.5rem;">
                        <h3 style="margin: 0 0 1rem 0; color: var(--neon-magenta);">🤖 AI Analysis</h3>
                        ${enrichment.summary ? `
                            <div style="margin-bottom: 1rem;">
                                <div style="font-size: 0.8rem; color: #888; margin-bottom: 0.25rem;">Summary</div>
                                <div style="color: #fff; line-height: 1.5;">${enrichment.summary}</div>
                            </div>
                        ` : ''}
                        ${enrichment.remediation_advice ? `
                            <div>
                                <div style="font-size: 0.8rem; color: #888; margin-bottom: 0.25rem;">Remediation Advice</div>
                                <div style="color: #fff; line-height: 1.5; white-space: pre-wrap;">${enrichment.remediation_advice}</div>
                            </div>
                        ` : ''}
                    </div>
                ` : ''}

                <!-- Severity Summary -->
                <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin-bottom: 1.5rem;">
                    <div style="background: rgba(255,68,68,0.1); border: 1px solid #ff4444; border-radius: 8px; padding: 1rem; text-align: center;">
                        <div style="font-size: 2rem; font-weight: bold; color: #ff4444;">${severityCount.CRITICAL}</div>
                        <div style="font-size: 0.8rem; color: #888;">Critical</div>
                    </div>
                    <div style="background: rgba(255,136,0,0.1); border: 1px solid #ff8800; border-radius: 8px; padding: 1rem; text-align: center;">
                        <div style="font-size: 2rem; font-weight: bold; color: #ff8800;">${severityCount.HIGH}</div>
                        <div style="font-size: 0.8rem; color: #888;">High</div>
                    </div>
                    <div style="background: rgba(255,221,0,0.1); border: 1px solid #ffdd00; border-radius: 8px; padding: 1rem; text-align: center;">
                        <div style="font-size: 2rem; font-weight: bold; color: #ffdd00;">${severityCount.MEDIUM}</div>
                        <div style="font-size: 0.8rem; color: #888;">Medium</div>
                    </div>
                    <div style="background: rgba(68,255,68,0.1); border: 1px solid #44ff44; border-radius: 8px; padding: 1rem; text-align: center;">
                        <div style="font-size: 2rem; font-weight: bold; color: #44ff44;">${severityCount.LOW}</div>
                        <div style="font-size: 0.8rem; color: #888;">Low</div>
                    </div>
                </div>

                <!-- Vulnerabilities List -->
                <div style="margin-bottom: 1.5rem;">
                    <h3 style="margin: 0 0 1rem 0; color: var(--neon-cyan);">🔍 Vulnerabilities (${vulnerabilities.length})</h3>
                    ${vulnerabilities.length === 0 ?
                '<div style="color: #888; text-align: center; padding: 2rem;">No vulnerabilities found in this report</div>' :
                `<div style="max-height: 400px; overflow-y: auto;">
                            ${vulnerabilities.map((v, i) => this.renderVulnerabilityItem(v, i)).join('')}
                        </div>`
            }
                </div>

                <!-- Metadata -->
                ${Object.keys(metadata).length > 0 ? `
                    <div style="background: rgba(0,0,0,0.3); border-radius: 8px; padding: 1rem;">
                        <h3 style="margin: 0 0 0.5rem 0; color: #888; font-size: 0.9rem;">📋 Metadata</h3>
                        <pre style="margin: 0; color: #aaa; font-size: 0.8rem; overflow-x: auto;">${JSON.stringify(metadata, null, 2)}</pre>
                    </div>
                ` : ''}
            </div>
        `;
    },

    renderVulnerabilityItem(vuln, index) {
        const severity = (vuln.severity || 'UNKNOWN').toUpperCase();
        const colors = {
            CRITICAL: '#ff4444',
            HIGH: '#ff8800',
            MEDIUM: '#ffdd00',
            LOW: '#44ff44',
            UNKNOWN: '#888'
        };
        const color = colors[severity] || colors.UNKNOWN;

        // Support both Trivy JSON field names (camelCase) and legacy snake_case
        const vulnId = vuln.vulnerabilityID || vuln.vulnerability_id || vuln.id || `vuln-${index}`;
        const pkgName = vuln.resource || vuln.pkg_name || vuln.package_name || 'Unknown Package';
        const installedVer = vuln.installedVersion || vuln.installed_version || vuln.version || 'N/A';
        const fixedVer = vuln.fixedVersion || vuln.fixed_version || vuln.fixed_in || null;
        const title = vuln.title || vuln.name || 'No title';
        const description = vuln.description || vuln.summary || '';
        const link = vuln.primaryLink || vuln.primary_link || null;

        return `
            <div style="background: rgba(0,0,0,0.3); border-left: 3px solid ${color}; margin-bottom: 0.5rem; border-radius: 0 4px 4px 0; overflow: hidden;">
                <div onclick="this.nextElementSibling.style.display = this.nextElementSibling.style.display === 'none' ? 'block' : 'none'" 
                     style="cursor: pointer; padding: 0.75rem; display: flex; align-items: center; gap: 0.75rem;">
                    <span style="background: ${color}; color: #000; padding: 0.15rem 0.5rem; border-radius: 3px; font-size: 0.7rem; font-weight: bold;">${severity}</span>
                    <span style="font-family: monospace; font-size: 0.8rem; color: var(--neon-cyan);">${link ? `<a href="${link}" target="_blank" style="color: var(--neon-cyan); text-decoration: none;">${vulnId} ↗</a>` : vulnId}</span>
                    <span style="flex: 1; color: #fff; font-size: 0.9rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${title}</span>
                    <span style="color: #888; font-size: 0.8rem;">▼</span>
                </div>
                <div style="display: none; padding: 0 0.75rem 0.75rem 0.75rem; border-top: 1px solid rgba(255,255,255,0.1);">
                    <div style="margin-top: 0.5rem;">
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 0.75rem; font-size: 0.85rem;">
                            <div><span style="color: #888;">Package:</span> <code>${pkgName}</code></div>
                            <div><span style="color: #888;">Installed:</span> <code>${installedVer}</code></div>
                        </div>
                        ${fixedVer ? `<div style="margin-bottom: 0.75rem; font-size: 0.85rem;"><span style="color: #888;">Fixed in:</span> <code style="color: #44ff44;">${fixedVer}</code></div>` : '<div style="margin-bottom: 0.5rem; font-size: 0.8rem; color: #888;">No fix available</div>'}
                        ${description ? `<div style="color: #ccc; font-size: 0.85rem; line-height: 1.5;">${description}</div>` : ''}
                    </div>
                </div>
            </div>
        `;
    },

    getScoreColor(score) {
        if (score >= 8) return '#ff4444';
        if (score >= 6) return '#ff8800';
        if (score >= 4) return '#ffdd00';
        return '#44ff44';
    },

    closeReportModal() {
        const modal = document.getElementById('report-detail-modal');
        if (modal) modal.style.display = 'none';
    },

    async loadReport(reportId) {
        if (!reportId) {
            this.fetchAndRender();
            return;
        }

        try {
            this.renderLoading();
            const vulns = await api.get(`/api/security/reports/${reportId}`);
            this.data = vulns;
            this.filteredData = { ...vulns };

            this.renderStats(vulns);
            this.renderChart(vulns);
            this.renderVulnerabilities(vulns);
            this.renderByNamespace(vulns);
            this.renderTopRisk(vulns);
            this.updateLastUpdated();

        } catch (error) {
            console.error('Failed to load report:', error);
            this.renderSecurityError(`Failed to load report: ${reportId}`);
        }
    },

    renderVulnerabilities(vulns) {
        const container = document.getElementById('security-vulns-content');
        const countEl = document.getElementById('security-vuln-count');

        if (!container) return;

        const images = vulns.images || [];
        if (countEl) countEl.textContent = images.length;

        if (images.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 3rem; text-align: center;">
                    <span style="font-size: 2rem;">✅</span>
                    <p>No vulnerabilities found matching your filters</p>
                </div>
            `;
            return;
        }

        // Sort by risk score (critical * 10 + high * 5 + medium * 2 + low)
        const sortedImages = [...images].sort((a, b) => {
            const scoreA = a.critical_count * 10 + a.high_count * 5 + a.medium_count * 2 + a.low_count;
            const scoreB = b.critical_count * 10 + b.high_count * 5 + b.medium_count * 2 + b.low_count;
            return scoreB - scoreA;
        });

        const table = `
            <table class="issues-table" style="font-size: 0.9rem;">
                <thead>
                    <tr>
                        <th style="text-align: left;">Image</th>
                        <th>Namespace</th>
                        <th style="text-align: center;">🔴</th>
                        <th style="text-align: center;">🟠</th>
                        <th style="text-align: center;">🟡</th>
                        <th style="text-align: center;">🟢</th>
                        <th>Score</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    ${sortedImages.map(img => {
            const riskScore = img.critical_count * 10 + img.high_count * 5 + img.medium_count * 2 + img.low_count;
            let riskClass = 'healthy';
            if (img.critical_count > 0) riskClass = 'unhealthy';
            else if (img.high_count > 0) riskClass = 'warning';
            else if (img.medium_count > 0) riskClass = 'info';

            return `
                            <tr class="vuln-row" data-image="${img.image}" style="cursor: pointer;" onclick="SecurityDashboard.toggleDetails('${this.escapeId(img.image)}')">
                                <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis;">
                                    <code title="${img.image}" style="font-size: 0.8rem;">${this.truncate(img.image, 45)}</code>
                                    <div style="font-size: 0.7rem; color: #888; margin-top: 2px;">${this.formatTime(img.last_scan)}</div>
                                </td>
                                <td><span class="status-badge info">${img.namespace}</span></td>
                                <td style="text-align: center;">${img.critical_count > 0 ? `<span class="status-badge unhealthy">${img.critical_count}</span>` : '-'}</td>
                                <td style="text-align: center;">${img.high_count > 0 ? `<span class="status-badge warning">${img.high_count}</span>` : '-'}</td>
                                <td style="text-align: center;">${img.medium_count > 0 ? `<span class="status-badge info">${img.medium_count}</span>` : '-'}</td>
                                <td style="text-align: center;">${img.low_count > 0 ? `<span class="status-badge healthy">${img.low_count}</span>` : '-'}</td>
                                <td><span class="status-badge ${riskClass}">${riskScore}</span></td>
                                <td>
                                    <button class="cyber-btn small" onclick="event.stopPropagation(); SecurityDashboard.scanImage('${img.image}')" title="Rescan">
                                        🔄
                                    </button>
                                </td>
                            </tr>
                            <tr id="details-${this.escapeId(img.image)}" style="display: none; background: rgba(0,0,0,0.3);">
                                <td colspan="8" style="padding: 1rem;">
                                    <div style="font-size: 0.85rem;">
                                        <strong>Full Image:</strong> <code>${img.image}</code><br>
                                        <strong>Digest:</strong> <code style="color: #888;">${img.digest || 'N/A'}</code><br>
                                        ${img.os ? `<strong>OS:</strong> ${img.os}<br>` : ''}
                                        ${img.scanner_version ? `<strong>Scanner:</strong> ${img.scanner_version}<br>` : ''}
                                    </div>
                                </td>
                            </tr>
                        `;
        }).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    renderByNamespace(vulns) {
        const container = document.getElementById('security-by-namespace');
        if (!container) return;

        const images = vulns.images || [];
        if (images.length === 0) {
            container.innerHTML = '<div class="no-issues">No data</div>';
            return;
        }

        // Group by namespace
        const byNs = {};
        images.forEach(img => {
            const ns = img.namespace || 'unknown';
            if (!byNs[ns]) {
                byNs[ns] = { count: 0, critical: 0, high: 0, medium: 0, low: 0, score: 0 };
            }
            byNs[ns].count++;
            byNs[ns].critical += img.critical_count || 0;
            byNs[ns].high += img.high_count || 0;
            byNs[ns].medium += img.medium_count || 0;
            byNs[ns].low += img.low_count || 0;

            // Calculate score addition
            byNs[ns].score += (img.critical_count || 0) * 10
                + (img.high_count || 0) * 5
                + (img.medium_count || 0) * 2
                + (img.low_count || 0);
        });

        const sorted = Object.entries(byNs).sort((a, b) => b[1].score - a[1].score);

        container.innerHTML = `
            <table class="issues-table" style="font-size: 0.85rem;">
                <thead>
                    <tr>
                        <th>Namespace</th>
                        <th style="text-align: center;">Images</th>
                        <th style="text-align: center;">🔴</th>
                        <th style="text-align: center;">🟠</th>
                        <th style="text-align: center;">Score</th>
                    </tr>
                </thead>
                <tbody>
                    ${sorted.map(([ns, data]) => {
            let scoreColor = '#44ff44';
            if (data.score > 50) scoreColor = '#ff4444';
            else if (data.score > 20) scoreColor = '#ff8800';
            else if (data.score > 5) scoreColor = '#ffdd00';

            return `
                        <tr>
                            <td><span class="status-badge info">${ns}</span></td>
                            <td style="text-align: center;">${data.count}</td>
                            <td style="text-align: center;">${data.critical > 0 ? `<span class="status-badge unhealthy">${data.critical}</span>` : '-'}</td>
                            <td style="text-align: center;">${data.high > 0 ? `<span class="status-badge warning">${data.high}</span>` : '-'}</td>
                            <td style="text-align: center; font-weight: bold; color: ${scoreColor};">${data.score}</td>
                        </tr>
                        `;
        }).join('')}
                </tbody>
            </table>
        `;
    },

    renderTopRisk(vulns) {
        const container = document.getElementById('security-top-risk');
        if (!container) return;

        const images = vulns.images || [];
        if (images.length === 0) {
            container.innerHTML = '<div class="no-issues">No data</div>';
            return;
        }

        // Sort by risk and take top 5
        const topRisk = [...images]
            .sort((a, b) => (b.critical_count * 10 + b.high_count * 5) - (a.critical_count * 10 + a.high_count * 5))
            .slice(0, 5);

        container.innerHTML = `
            <div style="padding: 0.5rem;">
                ${topRisk.map((img, i) => {
            const riskScore = img.critical_count * 10 + img.high_count * 5;
            let riskColor = '#44ff44';
            if (img.critical_count > 0) riskColor = '#ff4444';
            else if (img.high_count > 0) riskColor = '#ff8800';
            else if (img.medium_count > 0) riskColor = '#ffdd00';

            return `
                        <div style="display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem; border-bottom: 1px solid rgba(255,255,255,0.1); ${i === 0 ? 'background: rgba(255,0,0,0.1);' : ''}">
                            <span style="font-size: 1.2rem;">${i === 0 ? '🔥' : i < 3 ? '⚠️' : '⚡'}</span>
                            <div style="flex: 1; min-width: 0;">
                                <div style="font-size: 0.8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${img.image}">
                                    ${this.truncate(img.image, 35)}
                                </div>
                                <div style="font-size: 0.7rem; color: #888;">${img.namespace}</div>
                            </div>
                            <div style="text-align: right;">
                                <div style="font-size: 1rem; font-weight: bold; color: ${riskColor};">${riskScore}</div>
                                <div style="font-size: 0.7rem; color: #888;">risk score</div>
                            </div>
                        </div>
                    `;
        }).join('')}
            </div>
        `;
    },

    toggleDetails(imageId) {
        const row = document.getElementById(`details-${imageId}`);
        if (row) {
            row.style.display = row.style.display === 'none' ? 'table-row' : 'none';
        }
    },

    escapeId(str) {
        return str.replace(/[^a-zA-Z0-9]/g, '_');
    },

    async shadowScan(imageName) {
        // This is a placeholder for future image-specific scanning
        console.log('Individual scan requested for:', imageName);
    },

    async triggerScan() {
        const btn = document.getElementById('trivy-scan-btn');
        const originalText = btn ? btn.innerHTML : '';

        if (btn) {
            btn.disabled = true;
            btn.innerHTML = '<span>⏳</span> Scanning...';
        }

        try {
            const result = await api.post('/api/security/scan');
            console.log('Scan trigger result:', result);

            // Show notification
            if (window.utils && window.utils.showNotification) {
                window.utils.showNotification('Trivy scan triggered successfully', 'success');
            } else {
                alert('Trivy scan triggered successfully. It may take a few minutes to complete.');
            }

            // Refresh dashboard after a short delay
            setTimeout(() => this.refreshAll(), 5000);
        } catch (error) {
            console.error('Failed to trigger scan:', error);
            if (window.utils && window.utils.showNotification) {
                window.utils.showNotification('Failed to trigger scan: ' + (error.message || 'Unknown error'), 'error');
            } else {
                alert('Failed to trigger scan: ' + (error.message || 'Unknown error'));
            }
        } finally {
            if (btn) {
                btn.disabled = false;
                btn.innerHTML = originalText;
            }
        }
    },

    async scanImage(imageName) {
        // Trigger global scan for now as individual scan is complex with current setup
        console.log('Rescan requested for:', imageName);
        this.triggerScan();
    },

    exportCSV() {
        if (!this.filteredData || !this.filteredData.images) {
            alert('No data to export');
            return;
        }

        const images = this.filteredData.images;
        const headers = ['Image', 'Namespace', 'Critical', 'High', 'Medium', 'Low', 'Last Scan'];
        const rows = images.map(img => [
            img.image,
            img.namespace,
            img.critical_count,
            img.high_count,
            img.medium_count,
            img.low_count,
            img.last_scan
        ]);

        const csv = [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
        const blob = new Blob([csv], { type: 'text/csv' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `security-report-${new Date().toISOString().split('T')[0]}.csv`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    },

    truncate(text, length) {
        if (!text) return 'N/A';
        if (text.length <= length) return text;
        return text.substring(0, length) + '...';
    },

    formatTime(timestamp) {
        if (!timestamp) return 'N/A';
        try {
            const date = new Date(timestamp);
            const now = new Date();
            const diff = now - date;
            const hours = Math.floor(diff / (1000 * 60 * 60));

            if (hours < 1) return 'Just now';
            if (hours < 24) return `${hours}h ago`;
            if (hours < 168) return `${Math.floor(hours / 24)}d ago`;
            return date.toLocaleDateString();
        } catch (e) {
            return timestamp;
        }
    },

    // ========== Cilium Network Metrics ==========

    renderCiliumMetrics(metrics) {
        const container = document.getElementById('cilium-metrics-content');
        const countEl = document.getElementById('cilium-metrics-count');

        if (!container) return;

        // Handle empty or error case
        if (!metrics || !Array.isArray(metrics) || metrics.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">🌐</span>
                    <p>No Cilium network metrics available</p>
                    <p style="color: #888; font-size: 0.85rem; margin-top: 0.5rem;">
                        Metrics require Cilium CNI with Hubble enabled
                    </p>
                </div>
            `;
            if (countEl) countEl.textContent = '0';
            this.updateCiliumStats([]);
            return;
        }

        if (countEl) countEl.textContent = metrics.length;

        // Calculate totals for stats
        this.updateCiliumStats(metrics);

        // Sort by total bandwidth (ingress + egress)
        const sorted = [...metrics].sort((a, b) => {
            const totalA = (a.ingress_bytes_per_sec || 0) + (a.egress_bytes_per_sec || 0);
            const totalB = (b.ingress_bytes_per_sec || 0) + (b.egress_bytes_per_sec || 0);
            return totalB - totalA;
        });

        const table = `
            <table class="issues-table" style="font-size: 0.9rem;">
                <thead>
                    <tr>
                        <th style="text-align: left;">Namespace</th>
                        <th style="text-align: left;">Service</th>
                        <th style="text-align: right;">Ingress/s</th>
                        <th style="text-align: right;">Egress/s</th>
                        <th style="text-align: right;">Total/s</th>
                        <th style="text-align: center;">Connections</th>
                        <th>Utilization</th>
                    </tr>
                </thead>
                <tbody>
                    ${sorted.map(m => {
            const ingress = m.ingress_bytes_per_sec || 0;
            const egress = m.egress_bytes_per_sec || 0;
            const total = ingress + egress;
            const connections = m.connection_count || 0;

            // Calculate utilization bar (max 10MB/s for 100%)
            const maxBandwidth = 10 * 1024 * 1024; // 10 MB/s
            const utilization = Math.min((total / maxBandwidth) * 100, 100);
            let barColor = '#44ff44';
            if (utilization > 75) barColor = '#ff4444';
            else if (utilization > 50) barColor = '#ff8800';
            else if (utilization > 25) barColor = '#ffdd00';

            return `
                            <tr>
                                <td><span class="status-badge info">${m.namespace}</span></td>
                                <td><code style="font-size: 0.8rem;">${m.service}</code></td>
                                <td style="text-align: right; font-family: monospace;">${this.formatBytes(ingress)}/s</td>
                                <td style="text-align: right; font-family: monospace;">${this.formatBytes(egress)}/s</td>
                                <td style="text-align: right; font-family: monospace; font-weight: bold; color: var(--neon-cyan);">${this.formatBytes(total)}/s</td>
                                <td style="text-align: center;">
                                    <span class="status-badge ${connections > 100 ? 'warning' : 'healthy'}">${connections}</span>
                                </td>
                                <td style="width: 100px;">
                                    <div style="background: rgba(255,255,255,0.1); height: 8px; border-radius: 4px; overflow: hidden;">
                                        <div style="background: ${barColor}; height: 100%; width: ${utilization}%; transition: width 0.3s;"></div>
                                    </div>
                                </td>
                            </tr>
                        `;
        }).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    updateCiliumStats(metrics) {
        const data = metrics || [];
        const totalIngress = data.reduce((sum, m) => sum + (m.ingress_bytes_per_sec || 0), 0);
        const totalEgress = data.reduce((sum, m) => sum + (m.egress_bytes_per_sec || 0), 0);
        const totalConnections = data.reduce((sum, m) => sum + (m.connection_count || 0), 0);

        const ingressEl = document.getElementById('cilium-total-ingress');
        const egressEl = document.getElementById('cilium-total-egress');
        const connEl = document.getElementById('cilium-total-connections');
        const svcEl = document.getElementById('cilium-monitored-services');

        console.log('Updating Cilium stats:', { totalIngress, totalEgress, totalConnections, count: data.length });

        if (ingressEl) ingressEl.textContent = this.formatBytes(totalIngress);
        if (egressEl) egressEl.textContent = this.formatBytes(totalEgress);
        if (connEl) connEl.textContent = totalConnections.toLocaleString();
        if (svcEl) svcEl.textContent = data.length.toString();
    },

    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
};

// Global export
window.SecurityDashboard = SecurityDashboard;
