/**
 * KUSANAGI Utility Module
 * Handles Chat, World Clocks, Notifications, and general helpers
 */

// === CHAT FUNCTIONS ===
async function sendChatMessage() {
    const input = document.getElementById('chat-input');
    if (!input) return;

    const message = input.value.trim();
    if (!message) return;

    addChatMessage(message, 'user');
    input.value = '';

    const loadingId = 'loading-' + Date.now();
    addChatMessage('⏳ Thinking...', 'bot', loadingId);

    try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 180000);

        const response = await fetch('/api/chat', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                message,
                language: window.LocaleManager ? LocaleManager.currentLocale : 'en'
            }),
            signal: controller.signal
        });

        clearTimeout(timeoutId);
        if (!response.ok) throw new Error(`Server returned ${response.status}`);
        const data = await response.json();

        const loadingEl = document.getElementById(loadingId);
        if (loadingEl) loadingEl.remove();

        addChatMessage(data.response, 'bot');
    } catch (error) {
        const loadingEl = document.getElementById(loadingId);
        if (loadingEl) loadingEl.remove();

        const errorMsg = error.name === 'AbortError' ? '⚠️ Request timeout' : '❌ Error: ' + error.message;
        addChatMessage(errorMsg, 'bot');
    }
}

function addChatMessage(content, type, id) {
    const container = document.getElementById('chat-messages');
    if (!container) return;

    const messageDiv = document.createElement('div');
    messageDiv.className = `chat-message ${type}`;
    if (id) messageDiv.id = id;

    let html = content
        .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
        .replace(/`(.*?)`/g, '<code>$1</code>')
        .replace(/\n/g, '<br>')
        .replace(/^## (.*)/gm, '<h4>$1</h4>')
        .replace(/^- (.*)/gm, '• $1<br>');

    if (type === 'user') {
        messageDiv.innerHTML = `<div class="message-content"><strong>👤 You</strong><br>${html}</div>`;
    } else {
        const logo = '/static/images/logo.png';
        messageDiv.innerHTML = `<div class="message-content"><span class="bot-header"><img src="${logo}" class="chat-avatar" alt="Kusanagi"> <strong>Kusanagi</strong></span><br>${html}</div>`;
    }

    container.appendChild(messageDiv);
    container.scrollTop = container.scrollHeight;
}

function handleChatKeypress(event) {
    if (event.key === 'Enter') {
        sendChatMessage();
    }
}

// === EXPORT MENU ===
function toggleExportMenu() {
    const menu = document.getElementById('export-menu');
    if (menu) {
        menu.style.display = menu.style.display === 'none' ? 'block' : 'none';
    }
}

// Close export menu when clicking outside
document.addEventListener('click', (e) => {
    const trigger = document.querySelector('.export-trigger');
    const menu = document.getElementById('export-menu');
    if (menu && trigger && !trigger.contains(e.target) && !menu.contains(e.target)) {
        menu.style.display = 'none';
    }
});

// Global refresh trigger removed: consolidated in core.js

// Note: refreshAllKusanagiData is defined in core.js and exported there
window.sendChatMessage = sendChatMessage;
window.handleChatKeypress = handleChatKeypress;
window.toggleExportMenu = toggleExportMenu;

