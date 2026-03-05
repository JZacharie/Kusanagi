/**
 * StreamingManager - Manage movies from Cinestream
 */
const StreamingManager = {
    allMovies: [],
    filteredMovies: [],
    currentFilter: 'all', // 'all', 'French', 'Multi'
    searchQuery: '',

    init: function () {
        console.log("🎬 StreamingManager initialized");
    },

    /**
     * Called by TabManager when switching to the streaming tab
     */
    fetchMovies: async function () {
        const container = document.getElementById('streaming-container');
        if (!container) return;

        // Only show loading if we don't have data yet
        if (this.allMovies.length === 0) {
            container.innerHTML = '<div class="loading">Fetching movies from Cinestream...</div>';
        }

        try {
            const data = await api.get('/api/streaming');
            this.allMovies = data.items || [];
            this._updateStats(data);
            this.render();

            // Update last updated timestamp
            const updatedEl = document.getElementById('streaming-updated-at');
            if (updatedEl && data.cached_at) {
                updatedEl.textContent = new Date(data.cached_at).toLocaleString();
            }
        } catch (error) {
            console.error("❌ Failed to fetch streaming movies:", error);
            if (this.allMovies.length === 0) {
                container.innerHTML = `<div class="error">Failed to load movies: ${error.message}</div>`;
            }
            showNotification("Failed to load streaming data", "error");
        }
    },

    manualRefresh: async function () {
        const btn = document.getElementById('btn-streaming-refresh');
        if (btn) btn.classList.add('loading');

        try {
            showNotification("Refreshing streaming list...", "info");
            const data = await api.post('/api/streaming/refresh');
            this.allMovies = data.items || [];
            this._updateStats(data);
            this.render();

            const updatedEl = document.getElementById('streaming-updated-at');
            if (updatedEl && data.cached_at) {
                updatedEl.textContent = new Date(data.cached_at).toLocaleString();
            }
            showNotification("Streaming list refreshed", "success");
        } catch (error) {
            console.error("❌ Failed to refresh streaming movies:", error);
            showNotification("Failed to refresh streaming data", "error");
        } finally {
            if (btn) btn.classList.remove('loading');
        }
    },

    render: function () {
        const container = document.getElementById('streaming-container');
        if (!container) return;

        this._applyFilters();

        if (this.filteredMovies.length === 0) {
            container.innerHTML = '<div class="no-results">No movies found matching your criteria.</div>';
            return;
        }

        container.innerHTML = this.filteredMovies.map(movie => {
            const searchUrl = `https://thpibay.site/search/${encodeURIComponent(movie.title)}/1/99/0`;
            const escapedTitle = movie.title.replace(/"/g, '&quot;');

            return `
                <div class="movie-card">
                    <span class="badge badge-lang">${movie.language}</span>
                    <span class="badge badge-quality">${movie.quality}</span>
                    <img src="${movie.poster_url || '/static/images/no-poster.png'}" 
                         class="movie-poster" 
                         alt="${escapedTitle}" 
                         onerror="this.src='/static/images/no-poster.png'"
                         onclick="window.open('${movie.url}', '_blank')">
                    <div class="movie-info">
                        <div class="movie-title">${movie.title}</div>
                        <div class="movie-meta">
                            <span>${movie.year}</span>
                            <span style="opacity: 0.6; font-size: 0.75rem;">${movie.source}</span>
                        </div>
                        <div class="movie-genres" title="${movie.genres}">${movie.genres}</div>
                        <div class="movie-actions" style="margin-top: 0.75rem; display: flex; gap: 0.5rem;">
                            <a href="${movie.url}" target="_blank" class="cyber-btn btn-small" style="flex: 1; text-align: center; font-size: 0.7rem; padding: 0.3rem;">
                                🎬 VIEW
                            </a>
                            <a href="${searchUrl}" target="_blank" class="cyber-btn btn-small" style="flex: 1; text-align: center; font-size: 0.7rem; padding: 0.3rem;">
                                🔍 SEARCH
                            </a>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    },

    search: function (query) {
        this.searchQuery = query.toLowerCase().trim();
        this.render();
    },

    filter: function (filterType) {
        this.currentFilter = filterType;

        // Update button active states
        const btns = document.querySelectorAll('#streaming-filter-buttons .cyber-btn');
        btns.forEach(btn => {
            if (btn.id === `btn-streaming-${filterType.toLowerCase()}`) {
                btn.classList.add('active');
            } else {
                btn.classList.remove('active');
            }
        });

        this.render();
    },

    _applyFilters: function () {
        this.filteredMovies = this.allMovies.filter(movie => {
            // Search match
            const matchesSearch = !this.searchQuery ||
                movie.title.toLowerCase().includes(this.searchQuery) ||
                movie.year.toString().includes(this.searchQuery) ||
                movie.genres.toLowerCase().includes(this.searchQuery);

            // Category match
            let matchesCategory = true;
            if (this.currentFilter === 'French') {
                matchesCategory = movie.language.toLowerCase().includes('french') && !movie.language.toLowerCase().includes('multi');
            } else if (this.currentFilter === 'Multi') {
                matchesCategory = movie.language.toLowerCase().includes('multi');
            }

            return matchesSearch && matchesCategory;
        });
    },

    _updateStats: function (data) {
        const totalEl = document.getElementById('streaming-total');
        if (totalEl) totalEl.textContent = this.allMovies.length;
    }
};

window.StreamingManager = StreamingManager;
