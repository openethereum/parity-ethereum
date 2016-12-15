// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import solc from 'solc/browser-wrapper';
// import { isWebUri } from 'valid-url';
import registerPromiseWorker from 'promise-worker/register';

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

self.solcVersions = {};
self.files = {};

function handleMessage (message) {
  switch (message.action) {
    case 'compile':
      return compile(message.data);

    case 'load':
      return load(message.data);

    case 'setFiles':
      return setFiles(message.data);

    default:
      console.warn(`unknown action "${message.action}"`);
      return null;
  }
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

// @todo re-implement find imports (with ASYNC fetch)
// function findImports (path) {
//   if (self.files[path]) {
//     if (self.files[path].error) {
//       return Promise.reject(self.files[path].error);
//     }

//     return Promise.resolve(self.files[path]);
//   }

//   if (isWebUri(path)) {
//     console.log('[sw] fetching', path);

//     return fetch(path)
//       .then((r) => r.text())
//       .then((c) => {
//         console.log('[sw]', 'got content at ' + path);
//         self.files[path] = c;
//         return c;
//       })
//       .catch((e) => {
//         console.error('[sw]', 'fetching', path, e);
//         self.files[path] = { error: e };
//         throw e;
//       });
//   }

//   console.log(`[sw] path ${path} not found...`);
//   return Promise.reject('File not found');
// }

function compile (data, optimized = 1) {
  const { sourcecode, build } = data;

  return fetchSolidity(build)
    .then((compiler) => {
      const start = Date.now();
      console.log('[sw] compiling...');

      const input = {
        '': sourcecode
      };

      const compiled = compiler.compile({ sources: input }, optimized);

      const time = Math.round((Date.now() - start) / 100) / 10;
      console.log(`[sw] done compiling in ${time}s`);

      compiled.version = build.longVersion;

      return compiled;
    });
}

function load (build) {
  return fetchSolidity(build)
    .then(() => 'ok');
}

function fetchSolc (build) {
  const { path, longVersion } = build;
  const URL = `https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/bin/${path}`;

  return caches
    .match(URL)
    .then((response) => {
      if (response) {
        return response;
      }

      console.log(`[sw] fetching solc-bin ${longVersion} at ${URL}`);

      return fetch(URL)
        .then((response) => {
          if (!response || response.status !== 200 || response.type !== 'basic') {
            return response;
          }

          const responseToCache = response.clone();

          caches.open(CACHE_NAME)
            .then((cache) => {
              cache.put(URL, responseToCache);
            });

          return response;
        });
    });
}

function fetchSolidity (build) {
  const { path, longVersion } = build;

  if (self.solcVersions[path]) {
    return Promise.resolve(self.solcVersions[path]);
  }

  return fetchSolc(build)
    .then((r) => r.text())
    .then((code) => {
      const solcCode = code.replace(/^var Module;/, 'var Module=self.__solcModule;');
      self.__solcModule = {};

      console.log(`[sw] evaluating ${longVersion}`);

      // eslint-disable-next-line no-eval
      eval(solcCode);

      console.log(`[sw] done evaluating ${longVersion}`);

      const compiler = solc(self.__solcModule);
      self.solcVersions[path] = compiler;

      return compiler;
    });
}
