// Weather Dashboard Module
const WeatherDashboard = {
    refreshInterval: null,

    init() {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 1800000); // 30 minutes as per roadmap
        console.log('✅ Weather Dashboard initialized');
    },

    async fetchAndRender() {
        const container = document.getElementById('weather-content');
        if (!container) return;

        try {
            const response = await fetch('/api/weather/current');
            const data = await response.json();

            if (data.error) throw new Error(data.error);

            this.renderWeather(data);
        } catch (error) {
            console.error('Failed to fetch weather data:', error);
            container.innerHTML = `<div class="error">Failed to load weather data: ${error.message}</div>`;
        }
    },

    renderWeather(data) {
        const container = document.getElementById('weather-content');
        const cities = data.cities || [];

        if (cities.length === 0) {
            container.innerHTML = '<div class="no-news">No weather data available</div>';
            return;
        }

        let html = `
            <div class="weather-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-top: 1rem;">
                ${cities.map(city => `
                    <div class="weather-card" style="padding: 1.5rem; background: rgba(0, 255, 255, 0.05); border: 1px solid var(--neon-cyan); border-radius: 8px; position: relative; overflow: hidden;">
                        <div style="font-size: 3rem; position: absolute; top: 1rem; right: 1rem; opacity: 0.8;">${city.icon}</div>
                        <h3 style="margin: 0; font-size: 1.4rem; color: var(--neon-cyan);">${city.city}</h3>
                        <div style="font-size: 2.5rem; font-weight: bold; margin: 1rem 0;">${city.temp.toFixed(1)}°C</div>
                        <div style="text-transform: capitalize; font-size: 1.1rem; opacity: 0.9;">${city.description}</div>
                        
                        <div style="margin-top: 1.5rem; display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; font-size: 0.9rem; opacity: 0.7;">
                            <div>💧 Humidity: ${city.humidity}%</div>
                            <div>💨 Wind: ${city.wind_speed} km/h</div>
                        </div>
                        
                        <div style="margin-top: 1rem; font-size: 0.75rem; opacity: 0.5; text-align: right;">
                            Last updated: ${city.last_updated}
                        </div>
                    </div>
                `).join('')}
            </div>
            <div style="text-align: center; margin-top: 2rem; opacity: 0.4; font-size: 0.8rem;">
                Cached at: ${data.cached_at}
            </div>
        `;

        container.innerHTML = html;
    }
};

// Registered globally to be called by switchTab
window.WeatherDashboard = WeatherDashboard;
