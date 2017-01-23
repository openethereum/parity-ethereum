// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import * as actions from './signerActions';

import { inHex } from '~/api/format/input';
import { Signer } from '../../util/signer';

export default class SignerMiddleware {
  constructor (api) {
    this._api = api;
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

  onConfirmStart = (store, action) => {
    const { gas, gasPrice, id, password, payload, wallet } = action.payload;

    const handlePromise = (promise) => {
      promise
        .then((txHash) => {
          console.log('confirmRequest', id, txHash);

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

    // Sign request in-browser
    const transaction = payload.sendTransaction || payload.signTransaction;

    if (wallet && transaction) {
      const noncePromise = transaction.nonce.isZero()
        ? this._api.parity.nextNonce(transaction.from)
        : Promise.resolve(transaction.nonce);

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
        .all([ signerPromise, noncePromise ])
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
          return handlePromise(this._api.signer.confirmRequestRaw(id, rawTx));
        })
        .catch((error) => {
          console.error(error.message);
          store.dispatch(actions.errorConfirmRequest({ id, err: error.message }));
        });
    }

    handlePromise(this._api.signer.confirmRequest(id, { gas, gasPrice }, password));
  }

  onRejectStart = (store, action) => {
    const id = action.payload;

    this._api.signer
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
