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

    getSensorSVG(type, size = "20px") {
        const icons = {
            'humidity': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" class="weather-anim-sun">
                    <path d="M12 22s8-4 8-10A8 8 0 0 0 4 12c0 6 8 10 8 10z" stroke="var(--neon-cyan)" stroke-width="2"/>
                    <path d="M12 18s3-1.5 3-4" stroke="var(--neon-cyan)" stroke-width="1.5" opacity="0.6"/>
                </svg>`,
            'wind': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" class="weather-anim-cloud">
                    <path d="M9.59 4.59A2 2 0 1 1 11 8H2m10.59 11.41A2 2 0 1 0 14 16H2m15.73-8.27A2.5 2.5 0 1 1 19.5 12H2" stroke="var(--neon-magenta)" stroke-width="2" stroke-linecap="round"/>
                </svg>`,
            'pressure': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none">
                    <circle cx="12" cy="12" r="9" stroke="var(--neon-yellow)" stroke-width="2"/>
                    <path d="M12 7v5l3 3" stroke="var(--neon-yellow)" stroke-width="2" stroke-linecap="round" class="weather-anim-sun"/>
                </svg>`,
            'feels_like': `
                <svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" class="weather-anim-sun">
                    <path d="M14 14.76V3.5a2.5 2.5 0 0 0-5 0v11.26a4.5 4.5 0 1 0 5 0z" stroke="var(--neon-orange)" stroke-width="2"/>
                    <path d="M12 12v3" stroke="var(--neon-orange)" stroke-width="2" stroke-linecap="round"/>
                </svg>`
        };
        return icons[type] || '';
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
                    <div class="weather-card" style="position: relative; overflow: hidden; border-radius: 16px; padding: 2rem; display: flex; flex-direction: column; background: rgba(10, 20, 30, 0.6); border: 1px solid rgba(0, 255, 249, 0.15);">
                        <!-- Header with City & Main Icon -->
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem;">
                            <div>
                                <h2 style="margin: 0; font-size: 2.2rem; font-family: 'Rajdhani', sans-serif; letter-spacing: 2px; color: ${this.getTempColor(city.temp)}; text-shadow: 0 0 10px ${this.getTempColor(city.temp)}44;">${city.city.toUpperCase()}</h2>
                                <div style="text-transform: uppercase; font-size: 0.8rem; letter-spacing: 3px; opacity: 0.6; margin-top: 5px;">${city.description}</div>
                            </div>
                            <div style="filter: drop-shadow(0 0 15px rgba(0,255,249,0.3));">
                                ${this.getWeatherSVG(city.icon, "90px")}
                            </div>
                        </div>

                        <!-- Main Temp & Dynamic Sensors -->
                        <div style="display: flex; align-items: center; gap: 2rem; margin-bottom: 2.5rem; background: rgba(255,255,255,0.03); padding: 1.5rem; border-radius: 12px; border: 1px solid rgba(255,255,255,0.05);">
                            <div style="font-size: 4.8rem; font-weight: 700; font-family: 'Orbitron', sans-serif; color: #fff; line-height: 1;">${city.temp.toFixed(1)}<span style="font-size: 1.8rem; opacity: 0.4; vertical-align: super; margin-left: 5px;">°C</span></div>
                            
                            <div style="flex: 1; display: grid; grid-template-columns: 1fr; gap: 0.8rem;">
                                <div style="display: flex; align-items: center; justify-content: space-between; padding-bottom: 0.5rem; border-bottom: 1px solid rgba(255,255,255,0.05);">
                                    <div style="display: flex; align-items: center; gap: 10px;">
                                        ${this.getSensorSVG('humidity')}
                                        <span style="font-size: 0.75rem; letter-spacing: 1px; opacity: 0.5;">HUMIDITY</span>
                                    </div>
                                    <span style="font-family: 'Orbitron', sans-serif; font-weight: bold; color: var(--neon-cyan);">${city.humidity}%</span>
                                </div>
                                <div style="display: flex; align-items: center; justify-content: space-between; padding-bottom: 0.5rem; border-bottom: 1px solid rgba(255,255,255,0.05);">
                                    <div style="display: flex; align-items: center; gap: 10px;">
                                        ${this.getSensorSVG('wind')}
                                        <span style="font-size: 0.75rem; letter-spacing: 1px; opacity: 0.5;">WIND</span>
                                    </div>
                                    <span style="font-family: 'Orbitron', sans-serif; font-weight: bold; color: var(--neon-magenta);">${city.wind_speed}<span style="font-size: 0.6rem; margin-left: 2px;">KM/H</span></span>
                                </div>
                                <div style="display: flex; align-items: center; justify-content: space-between;">
                                    <div style="display: flex; align-items: center; gap: 10px;">
                                        ${this.getSensorSVG('feels_like')}
                                        <span style="font-size: 0.75rem; letter-spacing: 1px; opacity: 0.5;">FEELS LIKE</span>
                                    </div>
                                    <span style="font-family: 'Orbitron', sans-serif; font-weight: bold; color: var(--neon-yellow);">${city.feels_like.toFixed(1)}°</span>
                                </div>
                            </div>
                        </div>

                        <!-- 5-Day Forecast -->
                        <div style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 1rem; margin-top: auto;">
                            ${city.forecast.map(day => {
            // Robust date parsing
            const dateObj = new Date(day.date.replace(' ', 'T'));
            const weekday = isNaN(dateObj.getTime())
                ? (day.date.length > 5 ? day.date.substring(0, 3) : day.date)
                : dateObj.toLocaleDateString('en-US', { weekday: 'short' }).toUpperCase();

            return `
                                    <div class="forecast-day" style="padding: 1rem 0.5rem; text-align: center; background: rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px;">
                                        <div style="font-size: 0.65rem; font-weight: 800; opacity: 0.4; margin-bottom: 8px; letter-spacing: 1px;">${weekday}</div>
                                        <div style="margin-bottom: 8px; transform: scale(0.9);">
                                            ${this.getWeatherSVG(day.icon, "35px")}
                                        </div>
                                        <div style="font-size: 1.1rem; font-weight: 700; font-family: 'Orbitron', sans-serif; color: ${this.getTempColor(day.temp)};">${day.temp.toFixed(0)}°</div>
                                    </div>
                                `;
        }).join('')}
                        </div>
                        
                        <div style="margin-top: 2rem; display: flex; justify-content: space-between; align-items: center; opacity: 0.3; font-size: 0.6rem; font-family: 'JetBrains Mono', monospace; border-top: 1px dotted rgba(255,255,255,0.1); padding-top: 1rem;">
                            <span>STATUS: LIVE_FEED</span>
                            <span>SYNC_T: ${city.last_updated} // BUILD_01.25</span>
                        </div>
                    </div>
                `).join('')}
            </div>
            <div style="text-align: center; margin-top: 3rem; opacity: 0.2; font-size: 0.7rem; font-family: 'JetBrains Mono', monospace; text-transform: uppercase; letter-spacing: 3px;">
                Node synchronization complete // cache_ref: ${data.cached_at}
            </div>
        `;

        container.innerHTML = html;
    }
};

// Registered globally to be called by switchTab
window.WeatherDashboard = WeatherDashboard;

