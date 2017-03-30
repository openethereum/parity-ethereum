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

import dictionary from './dictionary';

// Allow a web worker in the browser, with a fallback for Node.js
const hasWebWorkers = typeof Worker !== 'undefined';
const KeyWorker = hasWebWorkers ? require('worker-loader!./worker')
                                : require('./worker').KeyWorker;

// Local accounts should never be used outside of the browser
export let keythereum = null;

if (hasWebWorkers) {
  require('keythereum/dist/keythereum');

  keythereum = window.keythereum;
}

export function phraseToAddress (phrase) {
  return phraseToWallet(phrase).then((wallet) => wallet.address);
}

export function phraseToWallet (phrase) {
  return new Promise((resolve, reject) => {
    const worker = new KeyWorker();

    worker.postMessage(phrase);
    worker.onmessage = ({ data }) => {
      resolve(data);
    };
  });
}

export function randomBytes (length) {
  if (keythereum) {
    return keythereum.crypto.randomBytes(length);
  }

  const buf = Buffer.alloc(length);

  for (let i = 0; i < length; i++) {
    buf[i] = Math.random() * 255;
  }

  return buf;
}

export function randomNumber (max) {
  // Use 24 bits to avoid the integer becoming signed via bitshifts
  const rand = randomBytes(3);

  const integer = (rand[0] << 16) | (rand[1] << 8) | rand[2];

  // floor to integer value via bitor 0
  return ((integer / 0xFFFFFF) * max) | 0;
}

export function randomWord () {
  // TODO mh: use better entropy
  const index = randomNumber(dictionary.length);

  return dictionary[index];
}

export function randomPhrase (length) {
  const words = [];

  while (length--) {
    words.push(randomWord());
  }

  return words.join(' ');
}
