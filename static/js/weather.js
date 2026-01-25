// Weather Dashboard Module
const WeatherDashboard = {
    refreshInterval: null,

    init() {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 300000); // 5 minutes
        console.log('✅ Weather Dashboard initialized (5min refresh)');
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

    getWeatherSVG(code, size = "80px") {
        const icons = {
            'sun': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" class="weather-anim-sun">
                    <circle cx="12" cy="12" r="5" fill="#FFD700" stroke="#FFA500" stroke-width="0.5"/>
                    <g stroke="#FFD700" stroke-width="2" stroke-linecap="round">
                        <line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/>
                        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                        <line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/>
                        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                    </g>
                </svg>`,
            'cloud': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" class="weather-anim-cloud">
                    <path d="M17.5 19c2.5 0 4.5-2 4.5-4.5S20 10 17.5 10c-.2 0-.4 0-.6.1C15.8 7.6 13.1 6 10 6 5.6 6 2 9.6 2 14s3.6 8 8 8h7.5c0-1.5 0-3 0-3z" fill="rgba(200, 240, 255, 0.8)"/>
                </svg>`,
            'rain': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none">
                    <path d="M17.5 15c2.5 0 4.5-2 4.5-4.5S20 6 17.5 6c-.2 0-.4 0-.6.1C15.8 3.6 13.1 2 10 2 5.6 2 2 5.6 2 10s3.6 8 8 8h7.5v-3z" fill="rgba(100, 150, 255, 0.6)" class="weather-anim-cloud"/>
                    <g stroke="#4FC3F7" stroke-width="2" stroke-linecap="round" class="weather-anim-rain">
                        <line x1="8" y1="19" x2="8" y2="21"/><line x1="12" y1="19" x2="12" y2="21"/><line x1="16" y1="19" x2="16" y2="21"/>
                    </g>
                </svg>`,
            'snow': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none">
                    <path d="M17.5 15c2.5 0 4.5-2 4.5-4.5S20 6 17.5 6c-.2 0-.4 0-.6.1C15.8 3.6 13.1 2 10 2 5.6 2 2 5.6 2 10s3.6 8 8 8h7.5v-3z" fill="rgba(255, 255, 255, 0.8)" class="weather-anim-cloud"/>
                    <g stroke="white" stroke-width="1.5" stroke-linecap="round" class="weather-anim-snow">
                        <circle cx="8" cy="19" r="0.5"/><circle cx="12" cy="21" r="0.5"/><circle cx="16" cy="19" r="0.5"/>
                    </g>
                </svg>`,
            'storm': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none">
                    <path d="M17.5 15c2.5 0 4.5-2 4.5-4.5S20 6 17.5 6c-.2 0-.4 0-.6.1C15.8 3.6 13.1 2 10 2 5.6 2 2 5.6 2 10s3.6 8 8 8h7.5v-3z" fill="#444" class="weather-anim-cloud"/>
                    <path d="M13 18l-2 3h3l-2 3" stroke="#FFEB3B" stroke-width="2" stroke-linejoin="round" class="weather-anim-bolt"/>
                </svg>`
        };

        const map = {
            '01d': 'sun', '01n': 'sun',
            '02d': 'cloud', '02n': 'cloud',
            '03d': 'cloud', '03n': 'cloud',
            '04d': 'cloud', '04n': 'cloud',
            '09d': 'rain', '09n': 'rain',
            '10d': 'rain', '10n': 'rain',
            '11d': 'storm', '11n': 'storm',
            '13d': 'snow', '13n': 'snow',
            '50d': 'cloud', '50n': 'cloud'
        };

        return icons[map[code] || 'sun'];
    },

    getTempColor(temp) {
        if (temp < 0) return 'var(--neon-blue, #007bff)';
        if (temp < 15) return 'var(--neon-cyan)';
        if (temp < 25) return 'var(--neon-yellow)';
        return 'var(--neon-orange, #ff8c00)';
    },

    renderWeather(data) {
        const container = document.getElementById('weather-content');
        const cities = data.cities || [];

        if (cities.length === 0) {
            container.innerHTML = '<div class="no-news">No weather data available</div>';
            return;
        }

        let html = `
            <div class="weather-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 2rem; padding: 1rem;">
                ${cities.map(city => `
                    <div class="weather-card" style="position: relative; overflow: hidden; border-radius: 16px; padding: 2rem; display: flex; flex-direction: column;">
                        <!-- Header with City & Main Icon -->
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;">
                            <div>
                                <h2 style="margin: 0; font-size: 2.2rem; font-family: 'Rajdhani', sans-serif; letter-spacing: 2px; color: ${this.getTempColor(city.temp)};">${city.city.toUpperCase()}</h2>
                                <div style="text-transform: uppercase; font-size: 0.8rem; letter-spacing: 3px; opacity: 0.6;">${city.description}</div>
                            </div>
                            <div style="filter: drop-shadow(0 0 10px rgba(0,255,249,0.3));">
                                ${this.getWeatherSVG(city.icon, "100px")}
                            </div>
                        </div>

                        <!-- Main Temp & Stats -->
                        <div style="display: flex; align-items: baseline; gap: 1rem; margin-bottom: 2rem;">
                            <div style="font-size: 4.5rem; font-weight: 700; font-family: 'Orbitron', sans-serif; color: #fff;">${city.temp.toFixed(1)}<span style="font-size: 2rem; opacity: 0.5;">°C</span></div>
                            <div style="flex: 1; display: flex; flex-direction: column; gap: 0.5rem; border-left: 1px solid rgba(255,255,255,0.1); padding-left: 1.5rem;">
                                <div style="display: flex; align-items: center; gap: 0.8rem;">
                                    <span style="font-size: 1.2rem;">💧</span>
                                    <span style="font-size: 0.9rem; opacity: 0.8; width: 80px;">HUMIDITY</span>
                                    <span style="font-weight: bold; color: var(--neon-cyan);">${city.humidity}%</span>
                                </div>
                                <div style="display: flex; align-items: center; gap: 0.8rem;">
                                    <span style="font-size: 1.2rem;">💨</span>
                                    <span style="font-size: 0.9rem; opacity: 0.8; width: 80px;">WIND</span>
                                    <span style="font-weight: bold; color: var(--neon-magenta);">${city.wind_speed} <span style="font-size: 0.7rem;">km/h</span></span>
                                </div>
                            </div>
                        </div>

                        <!-- 5-Day Forecast -->
                        <div style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 0.8rem; margin-top: auto;">
                            ${city.forecast.map(day => `
                                <div class="forecast-day" style="padding: 0.8rem 0.4rem; text-align: center; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.05);">
                                    <div style="font-size: 0.65rem; font-weight: 700; opacity: 0.5; margin-bottom: 0.5rem;">${new Date(day.date).toLocaleDateString('en-US', { weekday: 'short' }).toUpperCase()}</div>
                                    <div style="margin-bottom: 0.5rem;">
                                        ${this.getWeatherSVG(day.icon, "32px")}
                                    </div>
                                    <div style="font-size: 1.1rem; font-weight: 600; font-family: 'Orbitron', sans-serif; color: ${this.getTempColor(day.temp)};">${day.temp.toFixed(0)}°</div>
                                </div>
                            `).join('')}
                        </div>
                        
                        <div style="margin-top: 1.5rem; font-size: 0.65rem; opacity: 0.3; text-align: right; font-family: 'JetBrains Mono', monospace;">
                            SYNC_T: ${city.last_updated} // KUSANAGI_OS
                        </div>
                    </div>
                `).join('')}
            </div>
            <div style="text-align: center; margin-top: 2rem; opacity: 0.2; font-size: 0.7rem; font-family: 'JetBrains Mono', monospace; text-transform: uppercase; letter-spacing: 2px;">
                Global cache synchronized at: ${data.cached_at}
            </div>
        `;

        container.innerHTML = html;
    }
};

// Registered globally to be called by switchTab
window.WeatherDashboard = WeatherDashboard;

