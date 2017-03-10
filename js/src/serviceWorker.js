// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import registerPromiseWorker from 'promise-worker/register';
import { Signer } from '~/util/signer';
import SolidityUtils from '~/util/solidity';

const CACHE_NAME = 'parity-cache-v1';

registerPromiseWorker((msg) => {
  return handleMessage(msg);
});

self.addEventListener('install', (event) => {
  event.waitUntil(self.skipWaiting());
});

self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim());
});

self.addEventListener('fetch', (event) => {
  const { url } = event.request;

  if (/raw.githubusercontent.com\/ethereum\/solc-bin(.+)list\.json$/.test(url)) {
    // Return the cached version, but still update it in background
    return event.respondWith(cachedFetcher(event.request, true));
  }

  if (/raw.githubusercontent.com\/ethereum\/solc-bin(.+)soljson(.+)\.js$/.test(url)) {
    return event.respondWith(cachedFetcher(event.request));
  }
});

self.solc = {};
self.files = {};

function cachedFetcher (request, update = false) {
  return caches
    .match(request)
    .then((response) => {
      // Return cached response if exists and no
      // updates needed
      if (response && !update) {
        return response;
      }

      const fetcher = fetch(request.clone())
        .then((response) => {
          // Check if we received a valid response
          if (!response || response.status !== 200) {
            return response;
          }

          return caches
            .open(CACHE_NAME)
            .then((cache) => {
              cache.put(request, response.clone());
              return response;
            });
        });

      // Cache hit - return response
      // Still want to perform the fetch (update)
      if (response) {
        return response;
      }

      return fetcher;
    });
}

function handleMessage (message) {
  switch (message.action) {
    case 'compile':
      return compile(message.data);

    case 'load':
      return getCompiler(message.data).then(() => 'ok');

    case 'setFiles':
      return setFiles(message.data);

    case 'getSignerSeed':
      return getSignerSeed(message.data);

    default:
      console.warn(`unknown action "${message.action}"`);
      return null;
  }
}

function getSignerSeed (data) {
  console.log('deriving seed from service-worker');
  const { wallet, password } = data;

  return Signer.getSeed(wallet, password);
}

function compile (data) {
  const { build } = data;

  return getCompiler(build)
    .then((compiler) => {
      return SolidityUtils.compile(data, compiler);
    });
}

function setFiles (files) {
  const prevFiles = self.files;
  const nextFiles = files.reduce((obj, file) => {
    obj[file.name] = file.sourcecode;
    return obj;
  }, {});

  self.files = {
    ...prevFiles,
    ...nextFiles
  };

  return 'ok';
}

function getCompiler (build) {
  const { longVersion } = build;

  const fetcher = (url) => {
    const request = new Request(url);

    return cachedFetcher(request);
  };

  if (!self.solc[longVersion]) {
    self.solc[longVersion] = SolidityUtils
      .getCompiler(build, fetcher)
      .then((compiler) => {
        self.solc[longVersion] = compiler;
        return compiler;
      });
  }

  return Promise.resolve(self.solc[longVersion]);
}
