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

import { keccak_256 as keccak256 } from 'js-sha3';
import secp256k1 from 'secp256k1/js';

// Stay compatible between environments
if (typeof self !== 'object') {
  const scope = typeof global === 'undefined' ? window : global;

  scope.self = scope;
}

function bytesToHex (bytes) {
  return '0x' + Array.from(bytes).map(n => ('0' + n.toString(16)).slice(-2)).join('');
}

// Logic ported from /ethkey/src/brain.rs
function phraseToWallet (phrase) {
  let secret = keccak256.array(phrase);

  for (let i = 0; i < 16384; i++) {
    secret = keccak256.array(secret);
  }

  while (true) {
    secret = keccak256.array(secret);

    const secretBuf = Buffer.from(secret);

    if (secp256k1.privateKeyVerify(secretBuf)) {
      // No compression, slice out last 64 bytes
      const publicBuf = secp256k1.publicKeyCreate(secretBuf, false).slice(-64);
      const address = keccak256.array(publicBuf).slice(12);

      if (address[0] !== 0) {
        continue;
      }

      const wallet = {
        secret: bytesToHex(secretBuf),
        public: bytesToHex(publicBuf),
        address: bytesToHex(address)
      };

      return wallet;
    }
  }
}

self.onmessage = function ({ data }) {
  const wallet = phraseToWallet(data);

  postMessage(wallet);
  close();
};

// Emulate a web worker in Node.js
class KeyWorker {
  postMessage (data) {
    // Force async
    setTimeout(() => {
      const wallet = phraseToWallet(data);

      this.onmessage({ data: wallet });
    }, 0);
  }

  onmessage (event) {
    // no-op to be overriden
  }
}

if (exports != null) {
  exports.KeyWorker = KeyWorker;
}
