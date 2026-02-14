/**
 * Kusanagi Service Worker
 * Provides offline caching for PWA functionality
 */

const CACHE_NAME = 'kusanagi-v2';
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
    '/static/js/api.js',
    '/static/js/debug.js',
    '/static/js/ansi-parser.js',
    '/static/js/api-tracker.js',
    '/static/js/error-boundary.js',
    '/static/js/config.js',
    '/static/js/core.js',
    '/static/js/page-loader.js',
    '/static/js/k8s/state.js',
    '/static/js/k8s/pods.js',
    '/static/js/k8s/nodes.js',
    '/static/js/k8s/services.js',
    '/static/js/k8s/storage.js',
    '/static/js/k8s/argocd.js',
    '/static/js/k8s/main.js',
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

// Activate event - clean up old caches and notify clients
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
        }).then(() => {
            // Notify all clients to reload
            return self.clients.matchAll().then(clients => {
                clients.forEach(client => {
                    client.postMessage('reload');
                });
            });
        })
    );
    
    // Take control of all clients immediately
    self.clients.claim();
});

// Fetch event - network first for JS files, cache first for others
self.addEventListener('fetch', (event) => {
    // Skip non-GET requests
    if (event.request.method !== 'GET') return;
    
    // Skip API calls
    if (event.request.url.includes('/api/')) return;
    
    // For JS files: always try network first, then cache
    if (event.request.url.endsWith('.js')) {
        event.respondWith(
            fetch(event.request)
                .then(networkResponse => {
                    // Update cache with new version
                    if (networkResponse.status === 200) {
                        const responseToCache = networkResponse.clone();
                        caches.open(CACHE_NAME).then(cache => {
                            cache.put(event.request, responseToCache);
                        });
                    }
                    return networkResponse;
                })
                .catch(() => {
                    // Fallback to cache if network fails
                    return caches.match(event.request);
                })
        );
        return;
    }
    
    // For other files: cache first
    event.respondWith(
        caches.match(event.request)
            .then(response => {
                if (response) {
                    return response;
                }
                
                return fetch(event.request)
                    .then(networkResponse => {
                        if (!networkResponse || networkResponse.status !== 200) {
                            return networkResponse;
                        }
                        
                        const responseToCache = networkResponse.clone();
                        caches.open(CACHE_NAME)
                            .then(cache => {
                                cache.put(event.request, responseToCache);
                            });
                        
                        return networkResponse;
                    })
                    .catch(() => {
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
        console.log('⏩ Skipping waiting...');
        self.skipWaiting();
    }
});

console.log('📋 Service Worker loaded');
