import { isEqual } from 'lodash';
import logger from '../utils/logger';
import { updatePendingRequests, updateCompatibilityMode } from '../actions/requests';

export default class WsProvider {

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
    if (this.store.getState().requests.compatibilityMode) {
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

        logger.warn('[WS Provider] error fetching pending requests', err);
        return;
      }

      const txsStored = this.store.getState().requests.pending;
      if (isEqual(txsWs, txsStored)) {
        return;
      }

      logger.log('[WS Provider] requests changed ', txsWs);
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
        logger.warn('[WS Provider] error fetching pending transactions', err);
        return;
      }

      // Convert to new format
      txsWs = txsWs.map(transaction => {
        transaction.payload = {
          transaction: Object.assign({}, transaction.transaction)
        };
        return transaction;
      });

      const txsStored = this.store.getState().requests.pending;
      if (isEqual(txsWs, txsStored)) {
        return;
      }

      logger.log('[WS Provider] transactions changed ', txsWs);
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
