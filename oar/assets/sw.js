var cacheName = 'liveOAR-pwa-v1';
var filesToCache = [
  './',
  './index.html',
  './liveOAR.js',
  './liveOAR_bg.wasm',
];

self.addEventListener('install', function (e) {
  self.skipWaiting();
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Delete old caches on activate */
self.addEventListener('activate', function (e) {
  e.waitUntil(
    caches.keys().then(function (keys) {
      return Promise.all(
        keys.filter(function (key) { return key !== cacheName; })
            .map(function (key) { return caches.delete(key); })
      );
    }).then(function () { return self.clients.claim(); })
  );
});

/* Network first, cache as fallback */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    fetch(e.request)
      .then(function (response) {
        var clone = response.clone();
        caches.open(cacheName).then(function (cache) { cache.put(e.request, clone); });
        return response;
      })
      .catch(function () { return caches.match(e.request); })
  );
});
