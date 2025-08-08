// service_worker.js - Service Worker for offline-first Progressive Web App functionality

const CACHE_NAME = 'adaptive-learning-v1';
const RUNTIME_CACHE = 'runtime-cache-v1';
const DATA_CACHE = 'data-cache-v1';

// Files to cache immediately on install
const STATIC_CACHE_URLS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/css/app.css',
  '/js/app.js',
  '/wasm/app_bg.wasm',
  '/fonts/inter-regular.woff2',
  '/fonts/inter-bold.woff2',
  '/images/icon-192.png',
  '/images/icon-512.png',
  '/images/splash.png',
  '/offline.html'
];

// API endpoints that should be cached with network-first strategy
const API_CACHE_PATTERNS = [
  /\/api\/sessions\/.*/,
  /\/api\/learners\/.*/,
  /\/api\/domains\/.*/,
  /\/api\/metrics\/.*/
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
  console.log('[ServiceWorker] Installing...');

  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        console.log('[ServiceWorker] Caching static assets');
        return cache.addAll(STATIC_CACHE_URLS);
      })
      .then(() => {
        console.log('[ServiceWorker] Skip waiting');
        return self.skipWaiting();
      })
      .catch((error) => {
        console.error('[ServiceWorker] Installation failed:', error);
      })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('[ServiceWorker] Activating...');

  event.waitUntil(
    caches.keys()
      .then((cacheNames) => {
        return Promise.all(
          cacheNames
            .filter((cacheName) => {
              return cacheName !== CACHE_NAME &&
                     cacheName !== RUNTIME_CACHE &&
                     cacheName !== DATA_CACHE;
            })
            .map((cacheName) => {
              console.log('[ServiceWorker] Deleting old cache:', cacheName);
              return caches.delete(cacheName);
            })
        );
      })
      .then(() => {
        console.log('[ServiceWorker] Claiming clients');
        return self.clients.claim();
      })
  );
});

// Fetch event - serve from cache when offline
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip cross-origin requests
  if (url.origin !== self.location.origin) {
    return;
  }

  // Handle API requests with network-first strategy
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(handleApiRequest(request));
    return;
  }

  // Handle static assets with cache-first strategy
  event.respondWith(handleStaticRequest(request));
});

// Cache-first strategy for static assets
async function handleStaticRequest(request) {
  try {
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      // Update cache in background
      fetchAndCache(request, RUNTIME_CACHE);
      return cachedResponse;
    }

    const networkResponse = await fetch(request);

    // Cache successful responses
    if (networkResponse.ok) {
      const cache = await caches.open(RUNTIME_CACHE);
      cache.put(request, networkResponse.clone());
    }

    return networkResponse;
  } catch (error) {
    console.error('[ServiceWorker] Fetch failed:', error);

    // Return offline page for navigation requests
    if (request.mode === 'navigate') {
      const cache = await caches.open(CACHE_NAME);
      return cache.match('/offline.html');
    }

    // Return 503 for other requests
    return new Response('Service Unavailable', {
      status: 503,
      statusText: 'Service Unavailable'
    });
  }
}

// Network-first strategy for API requests
async function handleApiRequest(request) {
  const cache = await caches.open(DATA_CACHE);

  try {
    // Try network first
    const networkResponse = await fetch(request.clone());

    // Cache successful responses
    if (networkResponse.ok) {
      // Clone response before caching
      cache.put(request, networkResponse.clone());

      // Send message to client about successful sync
      broadcastMessage({
        type: 'SYNC_SUCCESS',
        url: request.url,
        timestamp: Date.now()
      });
    }

    return networkResponse;
  } catch (error) {
    console.log('[ServiceWorker] Network request failed, serving from cache');

    // Fallback to cache
    const cachedResponse = await cache.match(request);

    if (cachedResponse) {
      // Send message to client about offline mode
      broadcastMessage({
        type: 'SERVING_FROM_CACHE',
        url: request.url,
        timestamp: Date.now()
      });

      return cachedResponse;
    }

    // For POST/PUT requests that failed, queue for later sync
    if (request.method === 'POST' || request.method === 'PUT') {
      await queueForSync(request);

      // Return optimistic response
      return new Response(JSON.stringify({
        status: 'queued',
        message: 'Request queued for sync when online'
      }), {
        status: 202,
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Return error response
    return new Response(JSON.stringify({
      error: 'Network request failed and no cache available'
    }), {
      status: 503,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

// Helper function to fetch and cache in background
async function fetchAndCache(request, cacheName) {
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(cacheName);
      cache.put(request, response);
    }
  } catch (error) {
    // Silent fail for background updates
  }
}

// Queue failed requests for background sync
async function queueForSync(request) {
  const syncQueue = await getSyncQueue();

  const requestData = {
    url: request.url,
    method: request.method,
    headers: Object.fromEntries(request.headers.entries()),
    body: await request.text(),
    timestamp: Date.now()
  };

  syncQueue.push(requestData);
  await saveSyncQueue(syncQueue);

  // Register for background sync if available
  if ('sync' in self.registration) {
    await self.registration.sync.register('api-sync');
  }
}

// Background sync event handler
self.addEventListener('sync', (event) => {
  if (event.tag === 'api-sync') {
    console.log('[ServiceWorker] Background sync triggered');
    event.waitUntil(syncQueuedRequests());
  }
});

// Process queued requests when back online
async function syncQueuedRequests() {
  const syncQueue = await getSyncQueue();
  const failedRequests = [];

  for (const requestData of syncQueue) {
    try {
      const response = await fetch(requestData.url, {
        method: requestData.method,
        headers: requestData.headers,
        body: requestData.body
      });

      if (response.ok) {
        console.log('[ServiceWorker] Synced request:', requestData.url);

        // Notify client of successful sync
        broadcastMessage({
          type: 'SYNC_COMPLETE',
          url: requestData.url,
          timestamp: Date.now()
        });
      } else {
        failedRequests.push(requestData);
      }
    } catch (error) {
      console.error('[ServiceWorker] Sync failed for:', requestData.url);
      failedRequests.push(requestData);
    }
  }

  // Save failed requests back to queue
  await saveSyncQueue(failedRequests);

  return failedRequests.length === 0;
}

// Get sync queue from IndexedDB
async function getSyncQueue() {
  // In a real implementation, this would use IndexedDB
  // For now, using a simple in-memory store
  return self.syncQueue || [];
}

// Save sync queue to IndexedDB
async function saveSyncQueue(queue) {
  // In a real implementation, this would use IndexedDB
  self.syncQueue = queue;
}

// Broadcast message to all clients
function broadcastMessage(message) {
  self.clients.matchAll().then((clients) => {
    clients.forEach((client) => {
      client.postMessage(message);
    });
  });
}

// Handle messages from clients
self.addEventListener('message', (event) => {
  const { type, data } = event.data;

  switch (type) {
    case 'SKIP_WAITING':
      self.skipWaiting();
      break;

    case 'CLEAR_CACHE':
      clearAllCaches().then(() => {
        event.ports[0].postMessage({ success: true });
      });
      break;

    case 'CACHE_URLS':
      cacheUrls(data.urls).then(() => {
        event.ports[0].postMessage({ success: true });
      });
      break;

    case 'GET_CACHE_SIZE':
      getCacheSize().then((size) => {
        event.ports[0].postMessage({ size });
      });
      break;
  }
});

// Clear all caches
async function clearAllCaches() {
  const cacheNames = await caches.keys();
  await Promise.all(cacheNames.map(name => caches.delete(name)));
  console.log('[ServiceWorker] All caches cleared');
}

// Cache specific URLs on demand
async function cacheUrls(urls) {
  const cache = await caches.open(RUNTIME_CACHE);

  for (const url of urls) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        await cache.put(url, response);
      }
    } catch (error) {
      console.error('[ServiceWorker] Failed to cache:', url);
    }
  }
}

// Calculate total cache size
async function getCacheSize() {
  if ('estimate' in navigator.storage) {
    const estimate = await navigator.storage.estimate();
    return estimate.usage || 0;
  }
  return 0;
}

// Periodic cache cleanup (every 24 hours)
setInterval(async () => {
  const cache = await caches.open(DATA_CACHE);
  const requests = await cache.keys();
  const now = Date.now();
  const maxAge = 24 * 60 * 60 * 1000; // 24 hours

  for (const request of requests) {
    const response = await cache.match(request);
    const dateHeader = response.headers.get('date');

    if (dateHeader) {
      const responseTime = new Date(dateHeader).getTime();
      if (now - responseTime > maxAge) {
        await cache.delete(request);
        console.log('[ServiceWorker] Expired cache entry removed:', request.url);
      }
    }
  }
}, 24 * 60 * 60 * 1000);

// Handle push notifications (for practice reminders)
self.addEventListener('push', (event) => {
  const options = {
    body: event.data ? event.data.text() : 'Time for your daily practice!',
    icon: '/images/icon-192.png',
    badge: '/images/badge-72.png',
    vibrate: [100, 50, 100],
    data: {
      dateOfArrival: Date.now(),
      primaryKey: 1
    },
    actions: [
      {
        action: 'start',
        title: 'Start Practice',
        icon: '/images/checkmark.png'
      },
      {
        action: 'close',
        title: 'Later',
        icon: '/images/close.png'
      }
    ]
  };

  event.waitUntil(
    self.registration.showNotification('Adaptive Learning Reminder', options)
  );
});

// Handle notification clicks
self.addEventListener('notificationclick', (event) => {
  event.notification.close();

  if (event.action === 'start') {
    event.waitUntil(
      clients.openWindow('/training')
    );
  }
});

console.log('[ServiceWorker] Loaded successfully');
