self.addEventListener('install', function (e) {
  self.skipWaiting();
});

self.addEventListener('activate', function (e) {
  e.waitUntil(
    caches.keys().then(function (cacheNames) {
      return Promise.all(
        cacheNames.map(function (cache) {
          return caches.delete(cache);
        })
      );
    }).then(() => self.clients.claim())
  );
});

// ネットワーク接続が回復したらキャッシュを削除
self.addEventListener('message', function (event) {
  if (event.data === 'clearCache') {
    caches.keys().then(function (cacheNames) {
      return Promise.all(
        cacheNames.map(function (cache) {
          return caches.delete(cache);
        })
      );
    });
  }
});

// すべてのリクエストをネットワークから取得し、キャッシュは使用しない
self.addEventListener('fetch', function (e) {
  e.respondWith(fetch(e.request));
});
