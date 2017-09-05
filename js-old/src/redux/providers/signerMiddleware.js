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

import * as actions from './signerActions';

import { inHex } from '~/api/format/input';
import HardwareStore from '~/mobx/hardwareStore';
import { createSignedTx } from '~/util/qrscan';
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
        return this.confirmRawRequest(store, id, rawTx);
      });
  }

  confirmRawRequest (store, id, rawData) {
    const handlePromise = this._createConfirmPromiseHandler(store, id);

    return handlePromise(this._api.signer.confirmRequestRaw(id, rawData));
  }

  confirmSignedData (store, id, dataSigned) {
    const { signature } = dataSigned;

    return this.confirmRawRequest(store, id, signature);
  }

  confirmDecryptedMsg (store, id, decrypted) {
    const { msg } = decrypted;

    return this.confirmRawRequest(store, id, msg);
  }

  confirmSignedTransaction (store, id, txSigned) {
    const { netVersion } = store.getState().nodeStatus;
    const { signature, tx } = txSigned;
    const { rlp } = createSignedTx(netVersion, signature, tx);

    return this.confirmRawRequest(store, id, rlp);
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
        return this.confirmRawRequest(store, id, rawTx);
      })
      .catch((error) => {
        console.error(error.message);
        store.dispatch(actions.errorConfirmRequest({ id, err: error.message }));
      });
  }

  onConfirmStart = (store, action) => {
    const { condition, gas = 0, gasPrice = 0, id, password, payload, txSigned, dataSigned, decrypted, wallet } = action.payload;
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

    // TODO [ToDr] Support eth_sign for external wallet (wallet && dataSigned)
    if (dataSigned) {
      return this.confirmSignedData(store, id, dataSigned);
    }
    // TODO [ToDr] Support parity_decrypt for external wallet (wallet && decrypted)
    if (decrypted) {
      return this.confirmDecryptedMsg(store, id, decrypted);
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
