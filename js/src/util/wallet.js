// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import ethUtil from 'ethereumjs-util';
import scrypt from 'scryptsy';
import Transaction from 'ethereumjs-tx';
import crypto from 'crypto';
import aes from 'browserify-aes';

// Adapted from https://github.com/kvhnuke/etherwallet/blob/mercury/app/scripts/myetherwallet.js

export class Wallet {

  static fromJson (json, password) {
    if (json.version !== 3) {
      throw new Error('Only V3 wallets are supported');
    }

    const { kdf } = json.crypto;
    const kdfparams = json.crypto.kdfparams || {};
    const pwd = new Buffer(password);
    const salt = new Buffer(kdfparams.salt, 'hex');
    let derivedKey;

    if (kdf === 'scrypt') {
      derivedKey = scrypt(pwd, salt, kdfparams.n, kdfparams.r, kdfparams.p, kdfparams.dklen);
    } else if (kdf === 'pbkdf2') {
      if (kdfparams.prf !== 'hmac-sha256') {
        throw new Error('Unsupported parameters to PBKDF2');
      }
      derivedKey = crypto.pbkdf2Sync(pwd, salt, kdfparams.c, kdfparams.dklen, 'sha256');
    } else {
      throw new Error('Unsupported key derivation scheme');
    }

    const ciphertext = new Buffer(json.crypto.ciphertext, 'hex');
    const mac = ethUtil.sha3(Buffer.concat([derivedKey.slice(16, 32), ciphertext]));

    if (mac.toString('hex') !== json.crypto.mac) {
      throw new Error('Key derivation failed - possibly wrong passphrase');
    }

    const decipher = aes.createDecipheriv(
      json.crypto.cipher,
      derivedKey.slice(0, 16),
      new Buffer(json.crypto.cipherparams.iv, 'hex')
    );
    let seed = Buffer.concat([decipher.update(ciphertext), decipher.final()]);

    while (seed.length < 32) {
      const nullBuff = new Buffer([0x00]);
      seed = Buffer.concat([nullBuff, seed]);
    }

    return new Wallet(seed);
  }

  constructor (seed) {
    this.seed = seed;
  }

  signTransaction (transaction) {
    const tx = new Transaction(transaction);
    tx.sign(this.seed);
    return `0x${tx.serialize().toString('hex')}`;
  }

}
