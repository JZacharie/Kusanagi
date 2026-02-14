/**
 * Kusanagi PWA (Progressive Web App) Module
 * Handles service worker registration and install prompts
 */

// Register Service Worker with update handling
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/static/sw.js', { updateViaCache: 'none' })
            .then(registration => {
                console.log('✅ SW registered:', registration.scope);
                
                // Check for updates
                registration.addEventListener('updatefound', () => {
                    const newWorker = registration.installing;
                    console.log('🔄 New Service Worker found, installing...');
                    
                    newWorker.addEventListener('statechange', () => {
                        if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
                            console.log('🆕 New version available! Reload to update.');
                            // Force activation
                            newWorker.postMessage('skipWaiting');
                        }
                    });
                });
                
                // Force update check on load
                registration.update();
            })
            .catch(error => {
                console.log('❌ SW registration failed:', error);
            });
        
        // Listen for messages from SW
        navigator.serviceWorker.addEventListener('message', (event) => {
            if (event.data === 'reload') {
                console.log('🔄 Reloading for new version...');
                window.location.reload();
            }
        });
    });
}

// Clear all caches function
window.clearKusanagiCache = async function() {
    if ('caches' in window) {
        const cacheNames = await caches.keys();
        console.log('🗑️ Clearing caches:', cacheNames);
        await Promise.all(cacheNames.map(name => caches.delete(name)));
        console.log('✅ Caches cleared');
        
        // Unregister service worker
        if ('serviceWorker' in navigator) {
            const registration = await navigator.serviceWorker.getRegistration();
            if (registration) {
                await registration.unregister();
                console.log('✅ Service Worker unregistered');
            }
        }
        
        // Reload page
        window.location.reload(true);
    }
};

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

    // Wait for the user to respond to the prompt
    deferredPrompt.userChoice.then((choiceResult) => {
        if (choiceResult.outcome === 'accepted') {
            console.log('User accepted the install prompt');
        } else {
            console.log('User dismissed the install prompt');
        }
        deferredPrompt = null;
    });
}

// Hide install button after installation
window.addEventListener('appinstalled', () => {
    console.log('👍 PWA was installed');
    const installBtn = document.getElementById('pwa-install-item');
    if (installBtn) {
        installBtn.style.display = 'none';
    }
    deferredPrompt = null;
});
