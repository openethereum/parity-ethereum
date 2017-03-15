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

import * as actions from './signerActions';

import { inHex } from '~/api/format/input';
import HardwareStore from '~/mobx/hardwareStore';
import { Signer } from '~/util/signer';

export default class SignerMiddleware {
  constructor (api) {
    this._api = api;
    this._hwstore = HardwareStore.get(api);
  }

  toMiddleware () {
    return (store) => (next) => (action) => {
      let delegate;

      switch (action.type) {
        case 'signerStartConfirmRequest':
          delegate = this.onConfirmStart;
          break;

        case 'signerStartRejectRequest':
          delegate = this.onRejectStart;
          break;

        default:
          next(action);
          return;
      }

      if (!delegate) {
        return;
      }

      next(action);
      delegate(store, action);
    };
  }

  _createConfirmPromiseHandler (store, id) {
    return (promise) => {
      return promise
        .then((txHash) => {
          if (!txHash) {
            store.dispatch(actions.errorConfirmRequest({ id, err: 'Unable to confirm.' }));
            return;
          }

          store.dispatch(actions.successConfirmRequest({ id, txHash }));
        })
        .catch((error) => {
          console.error('confirmRequest', id, error);
          store.dispatch(actions.errorConfirmRequest({ id, err: error.message }));
        });
    };
  }

  createNoncePromise (transaction) {
    return !transaction.nonce || transaction.nonce.isZero()
      ? this._api.parity.nextNonce(transaction.from)
      : Promise.resolve(transaction.nonce);
  }

  confirmLedgerTransaction (store, id, transaction) {
    return this
      .createNoncePromise(transaction)
      .then((nonce) => {
        transaction.nonce = nonce;

        return this._hwstore.signLedger(transaction);
      })
      .then((rawTx) => {
        return this.confirmRawTransaction(store, id, rawTx);
      });
  }

  confirmRawTransaction (store, id, rawTx) {
    const handlePromise = this._createConfirmPromiseHandler(store, id);

    return handlePromise(this._api.signer.confirmRequestRaw(id, rawTx));
  }

  confirmSignedTransaction (store, id, txSigned) {
    const TEST = '{"id":"c2834685-bcce-1086-8cfc-46689463a41c","version":3,"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"fde79c1985186d17945ff65e9589e154"},"ciphertext":"d87f3251a1eeb8bfcb2d6c143d1bcf5d381872e45e1361075cb035192d2de33d","kdf":"pbkdf2","kdfparams":{"c":10240,"dklen":32,"prf":"hmac-sha256","salt":"cec82a9ed60793c7b1ee478884848bb2a11a796ce4daf6fa31d2e79dfa67ea49"},"mac":"253f1d24358342851191b1e1873cd95a8c5d001fecf54ab5ff09f9e789c6df33"},"address":"00d1efd527f8c41ce2910556433f4dc2672ce2a5","name":"test","meta":""}';
    const { rlp, signature, tx } = txSigned;

    const r = Buffer.from(signature.substr(2, 64), 'hex');
    const s = Buffer.from(signature.substr(66, 64), 'hex');

    // other payday pacific curtsy sulfur caramel suffix unvisited puppet monologue crusher
    // FIXME: First line is for replay protection, second without
    const v = Buffer.from([(parseInt(signature.substr(130, 2), 16) * 2) + 35]);
    // const v = Buffer.from([parseInt(signature.substr(130, 2), 16) + 27]);

    console.log('rlp', rlp);
    console.log('signature', signature);
    console.log('r, s, v', r.toString('hex'), s.toString('hex'), v.toString('hex'));

    const signedTx = new Transaction({
      to: tx.to,
      nonce: tx.nonce,
      gasPrice: tx.gasPrice,
      gasLimit: tx.gasLimit,
      value: tx.value,
      data: tx.data,
      chainId: tx._chainId,
      r,
      s,
      v
    });

    console.log('signedTx', signedTx);
    signedTx.verifySignature();

    Signer
      .fromJson(JSON.parse(TEST), '')
      .then((signer) => {
        signer.signTransactionObject(tx);

        console.log('Signer: r, s, v', tx.r.toString('hex'), tx.s.toString('hex'), tx.v.toString('hex'));

        tx.verifySignature();
      })
      .catch((error) => console.error('Signer', error));

    return this.confirmRawTransaction(store, id, signedTx.serialize().toString('hex'));
  }

  confirmWalletTransaction (store, id, transaction, wallet, password) {
    const { worker } = store.getState().worker;

    const signerPromise = worker && worker._worker.state === 'activated'
      ? worker
        .postMessage({
          action: 'getSignerSeed',
          data: { wallet, password }
        })
        .then((result) => {
          const seed = Buffer.from(result.data);

          return new Signer(seed);
        })
      : Signer.fromJson(wallet, password);

    // NOTE: Derving the key takes significant amount of time,
    // make sure to display some kind of "in-progress" state.
    return Promise
      .all([ signerPromise, this.createNoncePromise(transaction) ])
      .then(([ signer, nonce ]) => {
        const txData = {
          to: inHex(transaction.to),
          nonce: inHex(transaction.nonce.isZero() ? nonce : transaction.nonce),
          gasPrice: inHex(transaction.gasPrice),
          gasLimit: inHex(transaction.gas),
          value: inHex(transaction.value),
          data: inHex(transaction.data)
        };

        return signer.signTransaction(txData);
      })
      .then((rawTx) => {
        return this.confirmRawTransaction(store, id, rawTx);
      })
      .catch((error) => {
        console.error(error.message);
        store.dispatch(actions.errorConfirmRequest({ id, err: error.message }));
      });
  }

  onConfirmStart = (store, action) => {
    const { condition, gas = 0, gasPrice = 0, id, password, payload, txSigned, wallet } = action.payload;
    const handlePromise = this._createConfirmPromiseHandler(store, id);
    const transaction = payload.sendTransaction || payload.signTransaction;

    if (transaction) {
      const hardwareAccount = this._hwstore.wallets[transaction.from];

      if (wallet) {
        return this.confirmWalletTransaction(store, id, transaction, wallet, password);
      } else if (txSigned) {
        return this.confirmSignedTransaction(store, id, txSigned);
      } else if (hardwareAccount) {
        switch (hardwareAccount.via) {
          case 'ledger':
            return this.confirmLedgerTransaction(store, id, transaction);

          case 'parity':
          default:
            break;
        }
      }
    }

    return handlePromise(this._api.signer.confirmRequest(id, { gas, gasPrice, condition }, password));
  }

  onRejectStart = (store, action) => {
    const id = action.payload;

    return this._api.signer
      .rejectRequest(id)
      .then(() => {
        store.dispatch(actions.successRejectRequest({ id }));
      })
      .catch((error) => {
        console.error('rejectRequest', id, error);
        store.dispatch(actions.errorRejectRequest({ id, err: error.message }));
      });
  }
}
