"use strict";

var CACHE_NAME = 'doxle-v2';

// Cache app shell assets (HTML, JS, WASM, CSS) on first load
// API calls are never cached
self.addEventListener("install", function (event) {
  self.skipWaiting();
});

self.addEventListener("activate", function (event) {
  event.waitUntil(
    caches.keys().then(function (keys) {
      return Promise.all(
        keys
          .filter(function (key) { return key !== CACHE_NAME; })
          .map(function (key) { return caches.delete(key); })
      );
    }).then(function () {
      return self.clients.claim();
    })
  );
});

self.addEventListener("fetch", function (event) {
  var request = event.request;

  // Only cache GET requests
  if (request.method !== 'GET') return;

  var url = new URL(request.url);

  // Never cache API calls or external origins
  if (url.pathname.startsWith('/api') ||
      url.origin !== self.location.origin) {
    return;
  }

  // For app assets (.js, .wasm, .css, .svg, .woff2, .json) — cache-first
  // These have hashed filenames so cached versions are always correct
  var isAsset = url.pathname.startsWith('/assets/');

  if (isAsset) {
    event.respondWith(
      caches.match(request).then(function (cached) {
        if (cached) return cached;
        return fetch(request).then(function (response) {
          if (response.ok) {
            var clone = response.clone();
            caches.open(CACHE_NAME).then(function (cache) {
              cache.put(request, clone);
            });
          }
          return response;
        });
      })
    );
    return;
  }

  // For navigation (HTML) — network-first, fallback to cache
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .then(function (response) {
          if (response.ok) {
            var clone = response.clone();
            caches.open(CACHE_NAME).then(function (cache) {
              cache.put(request, clone);
            });
          }
          return response;
        })
        .catch(function () {
          return caches.match(request).then(function (cached) {
            return cached || caches.match('/index.html');
          });
        })
    );
    return;
  }
});
