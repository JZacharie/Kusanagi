/**
 * Kusanagi Theme Manager
 * Handles switching and persistence of UI themes
 */

const ThemeManager = {
    themes: {
        cyberpunk: {
            name: 'Cyberpunk',
            class: 'theme-cyberpunk',
            icon: '⚡'
        },
        modern: {
            name: 'Modern 2026',
            class: 'theme-modern',
            icon: '✨'
        },
        lootdrop: {
            name: 'Loot-Drop',
            class: 'theme-loot-drop',
            icon: '💎'
        },
        fundy: {
            name: 'Fundy Swiss',
            class: 'theme-fundy',
            icon: '🇨🇭'
        },
        white: {
            name: 'Pristine White',
            class: 'theme-white',
            icon: '⚪'
        },
        apple: {
            name: 'Apple Style',
            class: 'theme-apple',
            icon: '🍎'
        },
        notebook: {
            name: 'Notebook',
            class: 'theme-notebook',
            icon: '📒'
        }
    },

    storageKey: 'kusanagi_active_theme',
    prevThemeKey: 'kusanagi_prev_dark_theme',

    init() {
        const savedTheme = localStorage.getItem(this.storageKey) || 'cyberpunk';
        this.applyTheme(savedTheme);
        this.renderSwitcher();
        this.renderModeSwitch();
    },

    toggleMode() {
        const currentTheme = localStorage.getItem(this.storageKey) || 'cyberpunk';
        if (currentTheme === 'white') {
            // Switch back to previous dark theme
            const prevDark = localStorage.getItem(this.prevThemeKey) || 'cyberpunk';
            this.setTheme(prevDark);
        } else {
            // Save current dark theme and switch to white
            localStorage.setItem(this.prevThemeKey, currentTheme);
            this.setTheme('white');
        }
    },

    renderModeSwitch() {
        const container = document.getElementById('theme-mode-switch-container');
        if (!container) return;

        const currentTheme = localStorage.getItem(this.storageKey) || 'cyberpunk';
        const isWhite = currentTheme === 'white';

        container.innerHTML = `
            <button class="cyber-btn mode-toggle-btn" onclick="ThemeManager.toggleMode()" 
                    title="${isWhite ? 'Switch to Dark Mode' : 'Switch to Light Mode'}"
                    style="width: 40px; height: 40px; border-radius: 50%; padding: 0; display: flex; align-items: center; justify-content: center;">
                ${isWhite ? '🌙' : '☀️'}
            </button>
        `;
    },

    setTheme(themeName) {
        if (!this.themes[themeName]) return;
        localStorage.setItem(this.storageKey, themeName);
        this.applyTheme(themeName);
        this.renderSwitcher();
        this.renderModeSwitch();
    },

    applyTheme(themeName) {
        const theme = this.themes[themeName];

        // Remove all theme classes from body
        Object.values(this.themes).forEach(t => {
            document.body.classList.remove(t.class);
        });

        // Add active theme class
        document.body.classList.add(theme.class);

        // Update root attribute for specific CSS selection if needed
        document.documentElement.setAttribute('data-theme-name', themeName);

        // Update switcher UI if it exists
        const display = document.getElementById('active-theme-name');
        if (display) display.textContent = theme.name;
    },

    renderSwitcher() {
        const container = document.getElementById('theme-switcher-container');
        if (!container) return;

        const activeTheme = localStorage.getItem(this.storageKey) || 'cyberpunk';

        container.innerHTML = `
            <div class="theme-dropdown">
                <button class="cyber-btn theme-btn" onclick="ThemeManager.toggleMenu()">
                    <span id="active-theme-icon">${this.themes[activeTheme].icon}</span>
                    <span id="active-theme-name">${this.themes[activeTheme].name}</span>
                    <span class="chevron">▼</span>
                </button>
                <div id="theme-menu" class="theme-menu" style="display: none;">
                    ${Object.entries(this.themes).map(([key, theme]) => `
                        <div class="theme-option ${key === activeTheme ? 'active' : ''}" 
                             onclick="ThemeManager.setTheme('${key}'); ThemeManager.toggleMenu();">
                            <span class="option-icon">${theme.icon}</span>
                            <span class="option-name">${theme.name}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    },

    toggleMenu() {
        const menu = document.getElementById('theme-menu');
        if (menu) {
            menu.style.display = menu.style.display === 'none' ? 'block' : 'none';
        }
    }
};

// Handle clicks outside the menu
document.addEventListener('click', (e) => {
    const menu = document.getElementById('theme-menu');
    const btn = document.querySelector('.theme-btn');
    if (menu && btn && !menu.contains(e.target) && !btn.contains(e.target)) {
        menu.style.display = 'none';
    }
});

// Auto-init on script load
if (typeof document !== 'undefined') {
    document.addEventListener('DOMContentLoaded', () => ThemeManager.init());
}

window.ThemeManager = ThemeManager;
