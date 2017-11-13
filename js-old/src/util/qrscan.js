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

import Transaction from 'ethereumjs-tx';

import { inAddress, inHex, inNumber10 } from '~/api/format/input';
import { sha3 } from '~/api/util/sha3';

export function createUnsignedTx (api, netVersion, transaction) {
  const { data, from, gas, gasPrice, to, value } = transaction;

  return api.parity
    .nextNonce(from)
    .then((_nonce) => {
      const chainId = parseInt(netVersion, 10);
      const nonce = (!transaction.nonce || transaction.nonce.isZero())
        ? _nonce
        : transaction.nonce;

      const tx = new Transaction({
        chainId,
        data: inHex(data),
        gasPrice: inHex(gasPrice),
        gasLimit: inHex(gas),
        nonce: inHex(nonce),
        to: to ? inHex(to) : undefined,
        value: inHex(value),
        r: 0,
        s: 0,
        v: chainId
      });

      const rlp = inHex(tx.serialize().toString('hex'));
      const hash = sha3(rlp);

      return {
        chainId,
        hash,
        nonce,
        rlp,
        tx
      };
    });
}

export function createSignedTx (netVersion, signature, unsignedTx) {
  const chainId = parseInt(netVersion, 10);
  const { data, gasPrice, gasLimit, nonce, to, value } = unsignedTx;

  const r = Buffer.from(signature.substr(2, 64), 'hex');
  const s = Buffer.from(signature.substr(66, 64), 'hex');
  const v = Buffer.from([parseInt(signature.substr(130, 2), 16) + (chainId * 2) + 35]);

  const tx = new Transaction({
    chainId,
    data,
    gasPrice,
    gasLimit,
    nonce,
    to,
    value,
    r,
    s,
    v
  });

  return {
    chainId,
    rlp: inHex(tx.serialize().toString('hex')),
    tx
  };
}

export function generateQr (from, tx, hash, rlp) {
  if (tx.data && tx.data.length > 64) {
    return JSON.stringify({
      action: 'signTransactionHash',
      data: {
        account: from.substr(2),
        hash: hash.substr(2),
        details: {
          gasPrice: inNumber10(inHex(tx.gasPrice.toString('hex') || '0')),
          gas: inNumber10(inHex(tx.gasLimit.toString('hex') || '0')),
          nonce: inNumber10(inHex(tx.nonce.toString('hex') || '0')),
          to: inAddress(tx.to.toString('hex')),
          value: inHex(tx.value.toString('hex') || '0')
        }
      }
    });
  }

  return JSON.stringify({
    action: 'signTransaction',
    data: {
      account: from.substr(2),
      rlp: rlp.substr(2)
    }
  });
}

export function generateDataQr (data) {
  return Promise.resolve({
    data,
    value: JSON.stringify({
      action: 'signData',
      data
    })
  });
}

export function generateDecryptQr (data) {
  return Promise.resolve({
    decrypt: data,
    value: JSON.stringify({
      action: 'decrypt',
      data
    })
  });
}

export function generateTxQr (api, netVersion, transaction) {
  return createUnsignedTx(api, netVersion, transaction)
    .then((qr) => {
      qr.value = generateQr(transaction.from, qr.tx, qr.hash, qr.rlp);

      return qr;
    });
}
