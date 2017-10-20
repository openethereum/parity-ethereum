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

import { bytesToHex } from '~/api/util/format';
import { extern, slice } from './ethkey.js';

const isWorker = typeof self !== 'undefined';

// Stay compatible between environments
if (!isWorker) {
  const scope = typeof global === 'undefined' ? window : global;

  scope.self = scope;
}

// keythereum should never be used outside of the browser
let keythereum = require('keythereum');

if (isWorker) {
  keythereum = self.keythereum;
}

function route ({ action, payload }) {
  if (action in actions) {
    return actions[action](payload);
  }

  return null;
}

const input = slice(extern._input_ptr(), 1024);
const secret = slice(extern._secret_ptr(), 32);
const publicKey = slice(extern._public_ptr(), 64);
const address = slice(extern._address_ptr(), 20);

extern._ecpointg();

const actions = {
  phraseToWallet (phrase) {
    const phraseUtf8 = Buffer.from(phrase, 'utf8');

    if (phraseUtf8.length > input.length) {
      throw new Error('Phrase is too long!');
    }

    input.set(phraseUtf8);

    extern._brain(phraseUtf8.length);

    const wallet = {
      secret: bytesToHex(secret),
      public: bytesToHex(publicKey),
      address: bytesToHex(address)
    };

    return wallet;
  },

  verifySecret (key) {
    const keyBuf = Buffer.from(key.slice(2), 'hex');

    secret.set(keyBuf);

    return extern._verify_secret();
  },

  createKeyObject ({ key, password }) {
    key = Buffer.from(key);
    password = Buffer.from(password);

    const iv = keythereum.crypto.randomBytes(16);
    const salt = keythereum.crypto.randomBytes(32);
    const keyObject = keythereum.dump(password, key, salt, iv);

    return JSON.stringify(keyObject);
  },

  decryptPrivateKey ({ keyObject, password }) {
    password = Buffer.from(password);

    try {
      const key = keythereum.recover(password, keyObject);

      // Convert to array to safely send from the worker
      return Array.from(key);
    } catch (e) {
      return null;
    }
  }
};

self.onmessage = function ({ data }) {
  try {
    const result = route(data);

    postMessage([null, result]);
  } catch (err) {
    console.error(err);
    postMessage([err.toString(), null]);
  }
};

// Emulate a web worker in Node.js
class KeyWorker {
  postMessage (data) {
    // Force async
    setTimeout(() => {
      try {
        const result = route(data);

        this.onmessage({ data: [null, result] });
      } catch (err) {
        this.onmessage({ data: [err, null] });
      }
    }, 0);
  }

  onmessage (event) {
    // no-op to be overriden
  }
}

if (exports != null) {
  exports.KeyWorker = KeyWorker;
}
