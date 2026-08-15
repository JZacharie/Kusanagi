/**
 * Kusanagi - Applications Vue 360° Module
 */

const ApplicationsDashboard = {
    appsData: [],

    async init() {
        console.log("🛡️ ApplicationsDashboard: Initializing Vue 360°...");
        await this.fetchApplicationsData();
    },

    async fetchApplicationsData() {
        try {
            // Fetch from status page API or fallback to status dashboard endpoint
            const res = await fetch('https://jzacharie.github.io/jo3-status/index.html');
            if (res.ok) {
                const htmlText = await res.text();
                const jsonMatch = htmlText.match(/const rawApps = (\[[\s\S]*?\]);/);
                if (jsonMatch) {
                    this.appsData = JSON.parse(jsonMatch[1]);
                    console.log(`🛡️ ApplicationsDashboard: Loaded ${this.appsData.length} apps`);
                    this.updateStats();
                    this.populateProjects();
                    this.renderApps(this.appsData);
                    return;
                }
            }
        } catch (e) {
            console.warn("🛡️ ApplicationsDashboard: Direct fetch failed, trying local fallback", e);
        }

        // Fallback mock rendering if offline
        const grid = document.getElementById("kusanagiAppsGrid");
        if (grid) {
            grid.innerHTML = `<div style="grid-column: 1/-1; text-align: center; color: #9ca3af; padding: 2rem;">
                <p>Consultez la Vue 360° complète en direct :</p>
                <a href="https://jzacharie.github.io/jo3-status/" target="_blank" class="cyber-btn" style="margin-top: 1rem; display: inline-block;">🌐 Ouvrir le Dashboard 360° Status</a>
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

    cleanVer(v) {
        if (!v) return "latest";
        return v.toString().replace(/^v+/i, "");
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
            card.style.background = "rgba(17, 24, 39, 0.78)";
            card.style.border = "1px solid rgba(255, 255, 255, 0.08)";
            card.style.borderRadius = "12px";
            card.style.padding = "1.15rem";
            card.style.display = "flex";
            card.style.flexDirection = "column";
            card.style.justifySpaceBetween = "space-between";
            
            const ingressHtml = app.ingress_url ? 
                `<a href="${app.ingress_url}" target="_blank" rel="noopener noreferrer" style="display: flex; align-items: center; justify-content: center; gap: 0.5rem; width: 100%; padding: 0.6rem; background: rgba(99, 102, 241, 0.2); border: 1px solid rgba(99, 102, 241, 0.4); border-radius: 8px; color: #a5b4fc; font-weight: 600; font-size: 0.85rem; text-decoration: none;">🌐 Ouvrir (${app.ingress_url.replace("https://", "")})</a>` : 
                `<div style="display: flex; align-items: center; justify-content: center; width: 100%; padding: 0.6rem; background: rgba(255, 255, 255, 0.03); border: 1px dashed rgba(255, 255, 255, 0.1); border-radius: 8px; color: #9ca3af; font-size: 0.82rem;">Aucune route Ingress publique</div>`;

            const updateBadge = app.probe && app.probe.status === "UPDATE_AVAILABLE" ?
                `<span style="padding: 0.25rem 0.55rem; border-radius: 20px; font-size: 0.68rem; font-weight: 700; background: rgba(245, 158, 11, 0.2); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.4);">🚀 v${this.cleanVer(app.probe.latest)}</span>` : '';

            const statusClass = app.status === "Active" ? 
                `background: rgba(16, 185, 129, 0.2); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.4);` :
                `background: rgba(107, 114, 128, 0.2); color: #9ca3af; border: 1px solid rgba(107, 114, 128, 0.3);`;

            const gitleakColor = app.gitleaks_count > 0 ? '#ef4444' : '#10b981';

            card.innerHTML = `
                <div>
                    <div style="display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; margin-bottom: 0.85rem;">
                        <div style="display: flex; align-items: center; gap: 0.65rem; min-width: 0; flex: 1;">
                            <div style="width: 38px; height: 38px; border-radius: 8px; background: rgba(255, 255, 255, 0.06); display: flex; align-items: center; justify-content: center; flex-shrink: 0; border: 1px solid rgba(255, 255, 255, 0.1); overflow: hidden;">
                                <img src="${app.icon_url}" alt="${app.name}" style="width: 100%; height: 100%; object-fit: contain; padding: 4px;" onerror="ApplicationsDashboard.handleIconError(this, '${app.name}')">
                            </div>
                            <div style="font-size: 1.05rem; font-weight: 700; color: #fff; font-family: 'JetBrains Mono', monospace; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title="${app.name}">${app.name}</div>
                        </div>
                        <div style="display: flex; align-items: center; gap: 0.35rem; flex-shrink: 0;">
                            <span style="padding: 0.25rem 0.55rem; border-radius: 20px; font-size: 0.68rem; font-weight: 700; text-transform: uppercase; ${statusClass}">${app.status}</span>
                            ${updateBadge}
                        </div>
                    </div>

                    <div style="background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.06); border-radius: 8px; padding: 0.65rem; margin-bottom: 1rem; font-size: 0.82rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.25rem;">
                            <span style="color: #9ca3af;">Sonde Image :</span>
                            <span style="font-family: 'JetBrains Mono', monospace; font-weight:600; color: ${app.probe && app.probe.status === 'UPDATE_AVAILABLE' ? '#fbbf24' : '#10b981'};">
                                ${app.probe && app.probe.status === 'UPDATE_AVAILABLE' ? '🚀 v' + this.cleanVer(app.probe.current) + ' ➔ v' + this.cleanVer(app.probe.latest) : '✅ v' + this.cleanVer(app.probe ? app.probe.current : 'latest')}
                            </span>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <span style="color: #9ca3af;">Gitleaks / Passwords :</span>
                            <span style="font-family: 'JetBrains Mono', monospace; font-weight:600; color: ${gitleakColor};">
                                ${app.gitleaks_count > 0 ? '⚠️ ' + app.gitleaks_count + ' secrets bruts' : '🛡️ 0 leak (Vault)'}
                            </span>
                        </div>
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.4rem; font-size: 0.85rem; color: #9ca3af; margin-bottom: 1rem;">
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
                            <span style="color: #d1d5db; font-family: 'JetBrains Mono', monospace;" title="${app.chart}">${app.chart.length > 20 ? app.chart.substring(0, 17) + '...' : app.chart}</span>
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
