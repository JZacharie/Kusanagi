/**
 * Kusanagi Service Worker
 * Provides offline caching for PWA functionality
 */

const CACHE_NAME = 'kusanagi-v1';
const STATIC_ASSETS = [
    '/',
    '/static/css/cyberpunk.css',
    '/static/css/modern-2026.css',
    '/static/css/loot-drop.css',
    '/static/css/weather.css',
    '/static/css/homeassistant.css',
    '/static/css/sidebar-responsive.css',
    '/static/js/theme.js',
    '/static/js/utils.js',
    '/static/js/debug.js',
    '/static/js/ansi-parser.js',
    '/static/js/api-tracker.js',
    '/static/js/error-boundary.js',
    '/static/js/core.js',
    '/static/js/k8s.js',
    '/static/js/sidebar.js',
    '/static/js/pwa.js',
    '/static/images/logo.png',
    '/static/images/favicon.png'
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
    console.log('🔧 Service Worker installing...');
    
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => {
                console.log('📦 Caching static assets...');
                return cache.addAll(STATIC_ASSETS);
            })
            .catch(err => console.log('⚠️ Cache error:', err))
    );
    
    // Skip waiting to activate immediately
    self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
    console.log('🚀 Service Worker activating...');
    
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return Promise.all(
                cacheNames
                    .filter(name => name !== CACHE_NAME)
                    .map(name => {
                        console.log('🗑️ Deleting old cache:', name);
                        return caches.delete(name);
                    })
            );
        })
    );
    
    // Take control of all clients
    self.clients.claim();
});

// Fetch event - serve from cache or network
self.addEventListener('fetch', (event) => {
    // Skip non-GET requests
    if (event.request.method !== 'GET') return;
    
    // Skip API calls
    if (event.request.url.includes('/api/')) return;
    
    event.respondWith(
        caches.match(event.request)
            .then(response => {
                // Return cached version or fetch from network
                if (response) {
                    return response;
                }
                
                return fetch(event.request)
                    .then(networkResponse => {
                        // Don't cache non-successful responses
                        if (!networkResponse || networkResponse.status !== 200) {
                            return networkResponse;
                        }
                        
                        // Clone response for caching
                        const responseToCache = networkResponse.clone();
                        
                        caches.open(CACHE_NAME)
                            .then(cache => {
                                cache.put(event.request, responseToCache);
                            });
                        
                        return networkResponse;
                    })
                    .catch(() => {
                        // Return offline fallback if available
                        if (event.request.mode === 'navigate') {
                            return caches.match('/');
                        }
                    });
            })
    );
});

// Handle messages from client
self.addEventListener('message', (event) => {
    if (event.data === 'skipWaiting') {
        self.skipWaiting();
    }
});

console.log('📋 Service Worker loaded');
