/**
 * Kusanagi - Applications Vue 360° Module
 */

const ApplicationsDashboard = {
    appsData: [],
    loading: false,

    async init() {
        console.log("🛡️ ApplicationsDashboard: Initializing Vue 360°...");
        await this.fetchApplicationsData();
    },

    async fetchApplicationsData() {
        if (this.loading) return;
        this.loading = true;

        const grid = document.getElementById("kusanagiAppsGrid");
        if (grid && this.appsData.length === 0) {
            grid.innerHTML = `<div style="grid-column: 1/-1; text-align: center; color: var(--neon-cyan); padding: 3rem;">
                <div class="loading">Chargement des données 360° des applications...</div>
            </div>`;
        }

        // 1. Try internal Kusanagi Rust API: /api/applications/360
        try {
            const res = await fetch('/api/applications/360');
            if (res.ok) {
                const json = await res.json();
                if (json.data && Array.isArray(json.data) && json.data.length > 0) {
                    this.appsData = json.data;
                    this.updateStats();
                    this.populateProjects();
                    this.renderApps(this.appsData);
                    this.loading = false;
                    return;
                }
            }
        } catch (e) {
            console.warn("🛡️ /api/applications/360 not responding:", e);
        }

        // 2. Try fetching from the jo3-status hub
        try {
            const res = await fetch('https://jzacharie.github.io/jo3-status/index.html');
            if (res.ok) {
                const htmlText = await res.text();
                const jsonMatch = htmlText.match(/const rawApps = (\[[\s\S]*?\]);/);
                if (jsonMatch) {
                    this.appsData = JSON.parse(jsonMatch[1]);
                    this.updateStats();
                    this.populateProjects();
                    this.renderApps(this.appsData);
                    this.loading = false;
                    return;
                }
            }
        } catch (e) {
            console.warn("🛡️ Public status page fetch error:", e);
        }

        // 3. Try fallback to /api/argocd/status
        try {
            const res = await fetch('/api/argocd/status');
            if (res.ok) {
                const json = await res.json();
                const rawList = json.applications || (json.data && json.data.applications) || [];
                if (rawList.length > 0) {
                    this.appsData = rawList.map(a => ({
                        name: a.name || a.metadata?.name || 'app',
                        project: a.spec?.project || a.project || 'default',
                        namespace: a.spec?.destination?.namespace || a.namespace || '-',
                        chart: a.spec?.source?.chart || a.spec?.source?.path || a.chart || '-',
                        status: a.status?.health?.status === 'Healthy' ? 'Active' : 'Active',
                        badge_class: 'badge-active',
                        ingress_url: `https://${a.name}.p.zacharie.org`,
                        icon_url: `https://raw.githubusercontent.com/walkxcode/dashboard-icons/main/png/${a.name}.png`,
                        gitleaks_count: 0,
                        arch: 'amd64 (K8s)',
                        probe: { current: 'latest', latest: 'latest', status: 'UP_TO_DATE' }
                    }));
                    this.updateStats();
                    this.populateProjects();
                    this.renderApps(this.appsData);
                    this.loading = false;
                    return;
                }
            }
        } catch (e) {
            console.warn("🛡️ /api/argocd/status fallback error:", e);
        }

        this.loading = false;
        if (grid) {
            grid.innerHTML = `<div style="grid-column: 1/-1; text-align: center; color: #9ca3af; padding: 2rem;">
                <p>Consultez la Vue 360° complète en direct :</p>
                <a href="https://jzacharie.github.io/jo3-status/" target="_blank" class="cyber-btn" style="margin-top: 1rem; display: inline-block; padding: 0.6rem 1.2rem; text-decoration: none;">🌐 Ouvrir le Dashboard 360° Status</a>
            </div>`;
        }
    },

    updateStats() {
        const total = this.appsData.length;
        const active = this.appsData.filter(a => a.status === "Active").length;
        const updates = this.appsData.filter(a => a.probe && a.probe.status === "UPDATE_AVAILABLE").length;
        const gitleaks = this.appsData.reduce((acc, a) => acc + (a.gitleaks_count || 0), 0);

        if (document.getElementById("stat-total-apps")) document.getElementById("stat-total-apps").innerText = total;
        if (document.getElementById("stat-active-apps")) document.getElementById("stat-active-apps").innerText = active;
        if (document.getElementById("stat-updates-apps")) document.getElementById("stat-updates-apps").innerText = updates + " disp.";
        if (document.getElementById("stat-gitleaks-apps")) document.getElementById("stat-gitleaks-apps").innerText = gitleaks;
    },

    populateProjects() {
        const select = document.getElementById("appProjectFilter");
        if (!select) return;
        const projects = [...new Set(this.appsData.map(a => a.project))].sort();
        select.innerHTML = '<option value="ALL">Tous les projets</option>' + 
            projects.map(p => `<option value="${p}">${p}</option>`).join('');
    },

    formatVersion(ver) {
        if (!ver) return "latest";
        const str = ver.toString().trim();
        if (str.toLowerCase() === "latest" || str.toLowerCase() === "-" || str === "") {
            return "latest";
        }
        const clean = str.replace(/^[vV]+/, '');
        return "v" + clean;
    },

    handleIconError(img, appName) {
        const wrapper = img.parentElement;
        const clean = appName.replace(/^joe3-|^jo3-|-sbx|-dev|-prd|-vs/g, "");
        const words = clean.split(/[-_]/);
        let initials = "";
        if (words.length >= 2 && words[1].length > 0) {
            initials = (words[0][0] + words[1][0]).toUpperCase();
        } else {
            initials = clean.substring(0, 2).toUpperCase();
        }

        let hash = 0;
        for (let i = 0; i < appName.length; i++) {
            hash = appName.charCodeAt(i) + ((hash << 5) - hash);
        }
        const hue = Math.abs(hash % 360);

        wrapper.style.background = `linear-gradient(135deg, hsl(${hue}, 65%, 42%) 0%, hsl(${(hue + 50) % 360}, 75%, 26%) 100%)`;
        wrapper.style.color = '#ffffff';
        wrapper.style.fontFamily = "'JetBrains Mono', monospace";
        wrapper.style.fontSize = '0.9rem';
        wrapper.style.fontWeight = '700';
        wrapper.style.letterSpacing = '0.05em';
        wrapper.style.boxShadow = 'inset 0 1px 1px rgba(255,255,255,0.2)';
        wrapper.innerHTML = initials;
    },

    renderApps(apps) {
        const grid = document.getElementById("kusanagiAppsGrid");
        if (!grid) return;
        grid.innerHTML = "";

        if (apps.length === 0) {
            grid.innerHTML = `<div style="grid-column: 1/-1; text-align: center; color: #9ca3af; padding: 3rem;">Aucune application ne correspond à votre recherche.</div>`;
            return;
        }

        apps.forEach(app => {
            const card = document.createElement("div");
            card.className = "app-card";
            card.style.background = "rgba(17, 24, 39, 0.82)";
            card.style.border = "1px solid rgba(255, 255, 255, 0.08)";
            card.style.borderRadius = "14px";
            card.style.padding = "1.25rem";
            card.style.display = "flex";
            card.style.flexDirection = "column";
            card.style.justifyContent = "space-between";
            card.style.backdropFilter = "blur(14px)";
            
            const ingressHtml = app.ingress_url ? 
                `<a href="${app.ingress_url}" target="_blank" rel="noopener noreferrer" style="display: flex; align-items: center; justify-content: center; gap: 0.5rem; width: 100%; padding: 0.65rem; background: linear-gradient(135deg, rgba(99, 102, 241, 0.2) 0%, rgba(79, 70, 229, 0.3) 100%); border: 1px solid rgba(99, 102, 241, 0.4); border-radius: 8px; color: #a5b4fc; font-weight: 600; font-size: 0.85rem; text-decoration: none;">🌐 Ouvrir (${app.ingress_url.replace("https://", "")})</a>` : 
                `<div style="display: flex; align-items: center; justify-content: center; width: 100%; padding: 0.65rem; background: rgba(255, 255, 255, 0.03); border: 1px dashed rgba(255, 255, 255, 0.1); border-radius: 8px; color: #9ca3af; font-size: 0.82rem;">Aucune route Ingress publique</div>`;

            const updateBadge = (app.probe && app.probe.status === "UPDATE_AVAILABLE") ?
                `<span style="padding: 0.25rem 0.6rem; border-radius: 6px; font-size: 0.72rem; font-weight: 700; background: rgba(245, 158, 11, 0.2); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.4); white-space: nowrap;">🚀 Upgrade ${this.formatVersion(app.probe.latest)}</span>` : '';

            const statusClass = app.status === "Active" ? 
                `background: rgba(16, 185, 129, 0.2); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.4);` :
                `background: rgba(107, 114, 128, 0.2); color: #9ca3af; border: 1px solid rgba(107, 114, 128, 0.3);`;

            const gitleakColor = app.gitleaks_count > 0 ? '#ef4444' : '#10b981';

            const probeText = (app.probe && app.probe.status === "UPDATE_AVAILABLE") ?
                `🚀 ${this.formatVersion(app.probe.current)} ➔ ${this.formatVersion(app.probe.latest)}` :
                `✅ ${this.formatVersion(app.probe ? app.probe.current : 'latest')}`;

            card.innerHTML = `
                <div>
                    <div style="display: flex; flex-direction: column; gap: 0.45rem; margin-bottom: 0.85rem;">
                        <div style="display: flex; align-items: center; justify-content: space-between; gap: 0.6rem;">
                            <div style="display: flex; align-items: center; gap: 0.75rem; min-width: 0; flex: 1;">
                                <div style="width: 40px; height: 40px; border-radius: 10px; background: rgba(255, 255, 255, 0.06); display: flex; align-items: center; justify-content: center; flex-shrink: 0; border: 1px solid rgba(255, 255, 255, 0.1); overflow: hidden;">
                                    <img src="${app.icon_url}" alt="${app.name}" style="width: 100%; height: 100%; object-fit: contain; padding: 4px;" onerror="ApplicationsDashboard.handleIconError(this, '${app.name}')">
                                </div>
                                <div style="font-size: 1.15rem; font-weight: 700; color: #fff; font-family: 'JetBrains Mono', monospace; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title="${app.name}">${app.name}</div>
                            </div>
                            <span style="padding: 0.25rem 0.55rem; border-radius: 6px; font-size: 0.72rem; font-weight: 700; text-transform: uppercase; flex-shrink: 0; ${statusClass}">${app.status}</span>
                        </div>
                        ${updateBadge ? `<div style="display: flex; justify-content: flex-end;">${updateBadge}</div>` : ''}
                    </div>

                    <div style="background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.06); border-radius: 8px; padding: 0.75rem; margin-bottom: 1rem; font-size: 0.85rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.35rem;">
                            <span style="color: #9ca3af;">Sonde Image :</span>
                            <span style="font-family: 'JetBrains Mono', monospace; font-weight:600; color: ${app.probe && app.probe.status === 'UPDATE_AVAILABLE' ? '#fbbf24' : '#10b981'};">
                                ${probeText}
                            </span>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <span style="color: #9ca3af;">Gitleaks / Passwords :</span>
                            <span style="font-family: 'JetBrains Mono', monospace; font-weight:600; color: ${gitleakColor};">
                                ${app.gitleaks_count > 0 ? '⚠️ ' + app.gitleaks_count + ' secrets bruts' : '🛡️ 0 leak (Vault)'}
                            </span>
                        </div>
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.45rem; font-size: 0.88rem; color: #9ca3af; margin-bottom: 1rem;">
                        <div style="display: flex; justify-content: space-between;">
                            <span>Projet / NS :</span>
                            <span style="color: #d1d5db; font-family: 'JetBrains Mono', monospace;">${app.project} / ${app.namespace}</span>
                        </div>
                        <div style="display: flex; justify-content: space-between;">
                            <span>Architecture :</span>
                            <span style="color: #d1d5db; font-family: 'JetBrains Mono', monospace;">${app.arch || 'amd64 (K8s)'}</span>
                        </div>
                        <div style="display: flex; justify-content: space-between;">
                            <span>Source / Chart :</span>
                            <span style="color: #d1d5db; font-family: 'JetBrains Mono', monospace;" title="${app.chart}">${app.chart.length > 22 ? app.chart.substring(0, 19) + '...' : app.chart}</span>
                        </div>
                    </div>
                </div>
                <div>
                    ${ingressHtml}
                </div>
            `;
            grid.appendChild(card);
        });
    },

    quickFilter(type) {
        const statusFilter = document.getElementById("appStatusFilter");
        const searchInput = document.getElementById("appSearchInput");
        if (!statusFilter || !searchInput) return;

        if (type === "ALL") {
            statusFilter.value = "ALL";
            searchInput.value = "";
        } else if (type === "Active") {
            statusFilter.value = "Active";
        } else if (type === "Disabled") {
            statusFilter.value = "Disabled";
        } else if (type === "Updates") {
            statusFilter.value = "Update";
        } else if (type === "Gitleaks") {
            statusFilter.value = "ALL";
            searchInput.value = "secrets bruts";
        }
        this.filterApps();
    },

    filterApps() {
        const search = (document.getElementById("appSearchInput")?.value || "").toLowerCase();
        const project = document.getElementById("appProjectFilter")?.value || "ALL";
        const status = document.getElementById("appStatusFilter")?.value || "ALL";

        const filtered = this.appsData.filter(app => {
            const matchSearch = app.name.toLowerCase().includes(search) || 
                                app.project.toLowerCase().includes(search) || 
                                app.namespace.toLowerCase().includes(search) ||
                                (app.ingress_url && app.ingress_url.toLowerCase().includes(search)) ||
                                app.chart.toLowerCase().includes(search) ||
                                (search.includes("secret") && app.gitleaks_count > 0);
            const matchProject = (project === "ALL" || app.project === project);
            let matchStatus = true;
            if (status === "Active") matchStatus = app.status === "Active";
            else if (status === "Disabled") matchStatus = app.status === "Disabled";
            else if (status === "Update") matchStatus = app.probe && app.probe.status === "UPDATE_AVAILABLE";

            return matchSearch && matchProject && matchStatus;
        });

        this.renderApps(filtered);
    }
};

window.ApplicationsDashboard = ApplicationsDashboard;
