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

import workerPool from './workerPool';

export function createKeyObject (key, password) {
  return workerPool.action('createKeyObject', { key, password })
    .then((obj) => JSON.parse(obj));
}

export function decryptPrivateKey (keyObject, password) {
  return workerPool
    .action('decryptPrivateKey', { keyObject, password })
    .then((privateKey) => {
      if (privateKey) {
        return Buffer.from(privateKey);
      }

      return null;
    });
}

export function phraseToAddress (phrase) {
  return phraseToWallet(phrase)
    .then((wallet) => wallet.address);
}

export function phraseToWallet (phrase) {
  return workerPool.action('phraseToWallet', phrase);
}

export function verifySecret (secret) {
  return workerPool.action('verifySecret', secret);
}
