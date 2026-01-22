// Google Calendar OAuth2 Client-side Module
const CalendarOAuth = {
    async init() {
        this.updateUI();
        this.checkCallback();
    },

    updateUI() {
        const token = localStorage.getItem('google_calendar_token');
        const loginBtn = document.getElementById('btn-calendar-login');
        const logoutBtn = document.getElementById('btn-calendar-logout');

        if (token) {
            if (loginBtn) loginBtn.style.display = 'none';
            if (logoutBtn) logoutBtn.style.display = 'block';
        } else {
            if (loginBtn) loginBtn.style.display = 'block';
            if (logoutBtn) logoutBtn.style.display = 'none';
        }
    },

    async login() {
        console.log('Initiating Google OAuth2 flow via backend...');
        // Redirect to backend OAuth endpoint
        window.location.href = '/api/calendar/oauth/authorize';
    },

    logout() {
        localStorage.removeItem('google_calendar_token');
        this.updateUI();
        if (window.CalendarDashboard) window.CalendarDashboard.init();
    },

    checkCallback() {
        // Check if we're coming back from OAuth
        const urlParams = new URLSearchParams(window.location.search);
        const authStatus = urlParams.get('calendar_auth');

        if (authStatus === 'success') {
            // Extract token from hash
            const hash = window.location.hash.substring(1);
            const params = new URLSearchParams(hash);
            const token = params.get('access_token');

            if (token) {
                localStorage.setItem('google_calendar_token', token);
                console.log('✅ Google Calendar authenticated successfully');

                // Clean up URL
                window.history.replaceState({}, document.title, window.location.pathname);

                this.updateUI();
                if (window.CalendarDashboard) window.CalendarDashboard.init();
            }
        } else if (authStatus === 'error') {
            console.error('❌ Google Calendar authentication failed');
            alert('Failed to authenticate with Google Calendar. Please check your OAuth configuration.');
            window.history.replaceState({}, document.title, window.location.pathname);
        }
    },

    getToken() {
        return localStorage.getItem('google_calendar_token');
    }
};

// Global initializer
window.CalendarOAuth = CalendarOAuth;
