// Google Calendar OAuth2 Client-side Module
const CalendarOAuth = {
    clientId: '', // Will be fetched from backend or set via env
    redirectUri: window.location.origin + '/calendar-callback.html',
    scopes: 'https://www.googleapis.com/auth/calendar.readonly',

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
        // In a real app, we'd fetch the client ID from the backend
        // For development, we'll ask the user or use a mock flow
        console.log('Initiating Google OAuth2 flow...');

        const authUrl = `https://accounts.google.com/o/oauth2/v2/auth?` +
            `client_id=${this.clientId}&` +
            `redirect_uri=${encodeURIComponent(this.redirectUri)}&` +
            `response_type=token&` +
            `scope=${encodeURIComponent(this.scopes)}&` +
            `include_granted_scopes=true&` +
            `state=kusanagi_calendar`;

        // Mocking for now since we don't have a real Client ID yet
        if (!this.clientId) {
            const mockToken = prompt("Dev Mode: Enter a mock token or your real Google OAuth2 Token to test:");
            if (mockToken) {
                localStorage.setItem('google_calendar_token', mockToken);
                this.updateUI();
                if (window.CalendarDashboard) window.CalendarDashboard.init();
            }
            return;
        }

        window.location.href = authUrl;
    },

    logout() {
        localStorage.removeItem('google_calendar_token');
        this.updateUI();
        if (window.CalendarDashboard) window.CalendarDashboard.init();
    },

    checkCallback() {
        // Handle the fragment if redirected back from Google
        if (window.location.hash) {
            const params = new URLSearchParams(window.location.hash.substring(1));
            const token = params.get('access_token');
            if (token) {
                localStorage.setItem('google_calendar_token', token);
                window.location.hash = '';
                this.updateUI();
                if (window.CalendarDashboard) window.CalendarDashboard.init();
            }
        }
    },

    getToken() {
        return localStorage.getItem('google_calendar_token');
    }
};

// Global initializer
window.CalendarOAuth = CalendarOAuth;
