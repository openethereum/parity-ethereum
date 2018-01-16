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

import scrypt from 'scryptsy';
import Transaction from 'ethereumjs-tx';
import { pbkdf2Sync } from 'crypto';
import { createDecipheriv } from 'browserify-aes';

import { inHex } from '@parity/api/lib/format/input';
import { sha3 } from '@parity/api/lib/util/sha3';

// Adapted from https://github.com/kvhnuke/etherwallet/blob/mercury/app/scripts/myetherwallet.js

export class Signer {
  static fromJson (json, password) {
    return Signer
      .getSeed(json, password)
      .then((seed) => {
        return new Signer(seed);
      });
  }

  static getSeed (json, password) {
    try {
      const seed = Signer.getSyncSeed(json, password);

      return Promise.resolve(seed);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  static getSyncSeed (json, password) {
    if (json.version !== 3) {
      throw new Error('Only V3 wallets are supported');
    }

    const { kdf } = json.crypto;
    const kdfparams = json.crypto.kdfparams || {};
    const pwd = Buffer.from(password);
    const salt = Buffer.from(kdfparams.salt, 'hex');
    let derivedKey;

    if (kdf === 'scrypt') {
      derivedKey = scrypt(pwd, salt, kdfparams.n, kdfparams.r, kdfparams.p, kdfparams.dklen);
    } else if (kdf === 'pbkdf2') {
      if (kdfparams.prf !== 'hmac-sha256') {
        throw new Error('Unsupported parameters to PBKDF2');
      }

      derivedKey = pbkdf2Sync(pwd, salt, kdfparams.c, kdfparams.dklen, 'sha256');
    } else {
      throw new Error('Unsupported key derivation scheme');
    }

    const ciphertext = Buffer.from(json.crypto.ciphertext, 'hex');
    const mac = sha3(Buffer.concat([derivedKey.slice(16, 32), ciphertext]));

    if (mac !== inHex(json.crypto.mac)) {
      throw new Error('Key derivation failed - possibly wrong password');
    }

    const decipher = createDecipheriv(
      json.crypto.cipher,
      derivedKey.slice(0, 16),
      Buffer.from(json.crypto.cipherparams.iv, 'hex')
    );

    let seed = Buffer.concat([decipher.update(ciphertext), decipher.final()]);

    while (seed.length < 32) {
      const nullBuff = Buffer.from([0x00]);

      seed = Buffer.concat([nullBuff, seed]);
    }

    return seed;
  }

  constructor (seed) {
    this.seed = seed;
  }

  signTransactionObject (tx) {
    tx.sign(this.seed);

    return tx;
  }

  signTransaction (transaction) {
    const tx = new Transaction(transaction);

    return inHex(this.signTransactionObject(tx).serialize().toString('hex'));
  }
}
