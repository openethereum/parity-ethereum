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

import { isEqual } from 'lodash';
import { updatePendingRequests, updateCompatibilityMode } from '../actions/requests';

export default class WsDataProvider {
  constructor (store, ws) {
    this.store = store;
    this.ws = ws;
    this.ws.onOpen.push(::this.onWsOpen);
    this.ws.onMsg.push(::this.onWsMsg);
  }

  onWsOpen () {
    this.fetchPendingRequests();
  }

  onWsMsg (msg) {
    if (msg.data !== 'new_message') {
      return;
    }
    this.fetchPendingRequests();
  }

  fetchPendingRequests () {
    // TODO [legacy;todr] Remove
    if (this.store.getState().signerRequests.compatibilityMode) {
      return this.fetchPendingTransactionsFallback();
    }

    this.send('personal_requestsToConfirm', [], (err, txsWs) => {
      if (err) {
        // TODO [legacy;todr] Remove
        if (err.message === 'Method not found') {
          this.store.dispatch(updateCompatibilityMode(true));
          this.fetchPendingTransactionsFallback();
          return;
        }

        console.warn('[WS Provider] error fetching pending requests', err);
        return;
      }

      const txsStored = this.store.getState().signerRequests.pending;
      if (isEqual(txsWs, txsStored)) {
        return;
      }

      console.log('[WS Provider] requests changed ', txsWs);
      this.store.dispatch(updatePendingRequests(txsWs));
    });
  }

  // TODO [legacy;todr] Remove when we stop supporting beta
  fetchPendingTransactionsFallback () {
    this.send('personal_transactionsToConfirm', [], (err, txsWs) => {
      if (err) {
        if (err.message === 'Method not found') {
          this.store.dispatch(updateCompatibilityMode(false));
          this.fetchPendingRequests();
          return;
        }
        console.warn('[WS Provider] error fetching pending transactions', err);
        return;
      }

      // Convert to new format
      txsWs = txsWs.map(transaction => {
        transaction.payload = {
          transaction: Object.assign({}, transaction.transaction)
        };
        return transaction;
      });

      const txsStored = this.store.getState().signerRequests.pending;
      if (isEqual(txsWs, txsStored)) {
        return;
      }

      console.log('[WS Provider] transactions changed ', txsWs);
      this.store.dispatch(updatePendingRequests(txsWs));
    });
  }

  send (method, params, callback) {
    const payload = {
      jsonrpc: '2.0',
      method, params
    };
    this.ws.send(payload, callback);
  }

}
