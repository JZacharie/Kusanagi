// Calendar Dashboard Module
const CalendarDashboard = {
    refreshInterval: null,

    init() {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 300000); // 5 minutes
        console.log('✅ Calendar Dashboard initialized');
    },

    async fetchAndRender() {
        const container = document.getElementById('calendar-content');
        if (!container) return;

        try {
            const response = await fetch('/api/calendar/events');
            const data = await response.json();

            if (data.error) throw new Error(data.error);

            this.renderEvents(data);
        } catch (error) {
            console.error('Failed to fetch calendar data:', error);
            container.innerHTML = `<div class="error">Failed to load calendar data: ${error.message}</div>`;
        }
    },

    renderEvents(data) {
        const container = document.getElementById('calendar-content');
        const events = data.events || [];

        if (events.length === 0) {
            container.innerHTML = '<div class="no-news">No upcoming events found</div>';
            return;
        }

        let html = `
            <div class="calendar-list" style="display: flex; flex-direction: column; gap: 1rem; margin-top: 1rem;">
                ${events.map(event => {
            const startDate = new Date(event.start_time);
            const endDate = new Date(event.end_time);
            const isToday = startDate.toDateString() === new Date().toDateString();

            return `
                        <div class="event-card ${isToday ? 'today' : ''}" style="padding: 1.25rem; background: rgba(0, 255, 255, 0.03); border: 1px solid ${isToday ? 'var(--neon-green)' : 'rgba(0, 255, 255, 0.2)'}; border-left: 4px solid ${isToday ? 'var(--neon-green)' : 'var(--neon-cyan)'}; border-radius: 4px;">
                            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 0.5rem;">
                                <h3 style="margin: 0; font-size: 1.1rem; color: ${isToday ? 'var(--neon-green)' : 'var(--neon-cyan)'};">${event.summary}</h3>
                                <span class="status-badge ${event.status}" style="font-size: 0.7rem;">${event.status.toUpperCase()}</span>
                            </div>
                            
                            <div style="display: flex; gap: 1.5rem; font-size: 0.9rem; opacity: 0.8; margin-bottom: 0.5rem;">
                                <div>📅 ${startDate.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' })}</div>
                                <div>⏰ ${startDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })} - ${endDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</div>
                            </div>
                            
                            ${event.location ? `<div style="font-size: 0.85rem; opacity: 0.6; margin-bottom: 0.5rem;">📍 ${event.location}</div>` : ''}
                            ${event.description ? `<div style="font-size: 0.85rem; font-style: italic; opacity: 0.7; padding-top: 0.5rem; border-top: 1px dashed rgba(255,255,255,0.1);">${event.description}</div>` : ''}
                        </div>
                    `;
        }).join('')}
            </div>
            <div style="text-align: center; margin-top: 1.5rem; opacity: 0.4; font-size: 0.7rem;">
                Calendar: ${data.calendar_name} | Last updated: ${data.last_updated}
            </div>
        `;

        container.innerHTML = html;
    }
};

// Registered globally to be called by switchTab
window.CalendarDashboard = CalendarDashboard;
