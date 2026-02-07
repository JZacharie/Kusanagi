/**
 * Kusanagi PWA (Progressive Web App) Module
 * Handles service worker registration and install prompts
 */

// Register Service Worker
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/static/sw.js')
            .then(registration => {
                console.log('✅ SW registered:', registration.scope);
            })
            .catch(error => {
                console.log('❌ SW registration failed:', error);
            });
    });
}

// Handle Install Prompt for Android
let deferredPrompt;

function showInstallButton() {
    if (!deferredPrompt) return;
    const installBtn = document.getElementById('pwa-install-item');
    if (installBtn) {
        installBtn.style.display = 'block';
    }
}

window.addEventListener('beforeinstallprompt', (e) => {
    // Prevent the mini-infobar from appearing on mobile
    e.preventDefault();
    // Store the event for later use
    deferredPrompt = e;
    console.log('👍 PWA install prompt ready');

    // Attempt to show install button
    showInstallButton();
});

// Ensure button shows if event fired before DOM was ready
window.addEventListener('DOMContentLoaded', showInstallButton);

// Function to trigger install (can be called from a button)
function installPWA() {
    if (!deferredPrompt) {
        console.log('Install prompt not available');
        return;
    }

    // Show the install prompt
    deferredPrompt.prompt();

    // Wait for the user to respond
    deferredPrompt.userChoice.then((choiceResult) => {
        if (choiceResult.outcome === 'accepted') {
            console.log('✅ User accepted PWA install');
        } else {
            console.log('❌ User dismissed PWA install');
        }
        deferredPrompt = null;

        // Hide button
        const installBtn = document.getElementById('pwa-install-item');
        if (installBtn) {
            installBtn.style.display = 'none';
        }
    });
}

// Detect if app is installed
window.addEventListener('appinstalled', () => {
    console.log('✅ PWA was installed');
    deferredPrompt = null;
});

// Check display mode
function getDisplayMode() {
    const isStandalone = window.matchMedia('(display-mode: standalone)').matches;
    const isFullscreen = window.matchMedia('(display-mode: fullscreen)').matches;
    const isMinimalUi = window.matchMedia('(display-mode: minimal-ui)').matches;

    if (isFullscreen) return 'fullscreen';
    if (isStandalone) return 'standalone';
    if (isMinimalUi) return 'minimal-ui';
    return 'browser';
}

// Log display mode
console.log('📱 Display mode:', getDisplayMode());

// Export for global use
window.installPWA = installPWA;
window.getDisplayMode = getDisplayMode;
