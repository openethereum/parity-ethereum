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

import { inHex } from '~/api/format/input';

export function createUnsignedTx (api, netVersion, gasStore, transaction) {
  const { data, from, gas, gasPrice, to, value } = gasStore.overrideTransaction(transaction);

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
        to: inHex(to),
        value: inHex(value),
        r: 0,
        s: 0,
        v: chainId
      });

      return {
        chainId,
        nonce,
        rlp: inHex(tx.serialize().toString('hex')),
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
