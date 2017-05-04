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

import toolbox from 'sw-toolbox';

/**
 * Cache the SOLC files : if not available, make a network request
 */
toolbox.router.any(/rawgit.com\/ethereum\/solc-bin(.+)list\.json$/, toolbox.networkFirst);
toolbox.router.any(/rawgit.com\/ethereum\/solc-bin(.+)soljson(.+)\.js$/, toolbox.cacheFirst);

self.addEventListener('install', (event) => {
  event.waitUntil(self.skipWaiting());
});

self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim());
});
