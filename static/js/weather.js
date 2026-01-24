// Weather Dashboard Module
const WeatherDashboard = {
    refreshInterval: null,

    init() {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 3600000); // 1 hour
        console.log('✅ Weather Dashboard initialized (hourly refresh)');
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

    getAnimatedIcon(code) {
        const iconMap = {
            '01d': '☀️', '01n': '🌙',
            '02d': '⛅', '02n': '🌥️',
            '03d': '☁️', '03n': '☁️',
            '04d': '☁️', '04n': '☁️',
            '09d': '🌧️', '09n': '🌧️',
            '10d': '🌦️', '10n': '🌦️',
            '11d': '⚡', '11n': '⚡',
            '13d': '❄️', '13n': '❄️',
            '50d': '🌫️', '50n': '🌫️'
        };

        const animationClass = {
            '01d': 'pulse-anim',
            '01n': 'float-anim',
            '02d': 'float-anim',
            '03d': 'float-anim',
            '04d': 'float-anim',
            '09d': 'rain-anim',
            '10d': 'rain-anim',
            '11d': 'flash-anim',
            '13d': 'float-anim'
        };

        const emoji = iconMap[code] || '🌡️';
        const cssClass = animationClass[code] || '';

        return `<div class="weather-icon-anim ${cssClass}" style="font-size: 3rem; display: flex; align-items: center; justify-content: center;">${emoji}</div>`;
    },

    renderWeather(data) {
        const container = document.getElementById('weather-content');
        const cities = data.cities || [];

        if (cities.length === 0) {
            container.innerHTML = '<div class="no-news">No weather data available</div>';
            return;
        }

        let html = `
            <div class="weather-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 1.5rem; margin-top: 1rem;">
                ${cities.map(city => `
                    <div class="weather-card" style="padding: 1.5rem; background: rgba(0, 255, 249, 0.05); border: 1px solid var(--neon-cyan); border-radius: 12px; position: relative;">
                        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1rem;">
                            <div>
                                <h3 style="margin: 0; font-size: 1.6rem; color: var(--neon-cyan);">${city.city}</h3>
                                <div style="text-transform: capitalize; font-size: 1.1rem; opacity: 0.9;">${city.description}</div>
                            </div>
                            ${this.getAnimatedIcon(city.icon)}
                        </div>

                        <div style="font-size: 3rem; font-weight: bold; margin: 0.5rem 0;">${city.temp.toFixed(1)}°C</div>
                        
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; font-size: 0.9rem; opacity: 0.7; margin-bottom: 1.5rem;">
                            <div>💧 Humidity: ${city.humidity}%</div>
                            <div>💨 Wind: ${city.wind_speed} km/h</div>
                        </div>

                        <div class="forecast-grid">
                            ${city.forecast.map(day => `
                                <div class="forecast-day">
                                    <div style="font-size: 0.7rem; opacity: 0.6; margin-bottom: 0.2rem;">${new Date(day.date).toLocaleDateString('en-US', { weekday: 'short' })}</div>
                                    <div style="font-size: 1.2rem; margin: 0.3rem 0;">${this.getAnimatedIcon(day.icon).replace('3rem', '1.2rem')}</div>
                                    <div class="forecast-temp">${day.temp.toFixed(0)}°</div>
                                </div>
                            `).join('')}
                        </div>
                        
                        <div style="margin-top: 1rem; font-size: 0.7rem; opacity: 0.4; text-align: right;">
                            Last update: ${city.last_updated}
                        </div>
                    </div>
                `).join('')}
            </div>
            <div style="text-align: center; margin-top: 2rem; opacity: 0.4; font-size: 0.8rem;">
                Server cache: ${data.cached_at}
            </div>
        `;

        container.innerHTML = html;
    }
};

// Registered globally to be called by switchTab
window.WeatherDashboard = WeatherDashboard;
