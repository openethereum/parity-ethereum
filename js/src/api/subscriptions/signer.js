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

import { outTransaction } from '../format/output';

export default class Signer {
  constructor (updateSubscriptions, api, subscriber) {
    this._subscriber = subscriber;
    this._api = api;
    this._updateSubscriptions = updateSubscriptions;
    this._started = false;

    // Try to restart subscription if transport is closed
    this._api.transport.on('close', () => {
      if (this.isStarted) {
        this.start();
      }
    });
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    if (this._api.isPubSub) {
      const subscription = this._api.pubsub
        .subscribeAndGetResult(
          callback => this._api.pubsub.signer.pendingRequests(callback),
          requests => {
            this.updateSubscriptions(requests);
            return requests;
          }
        );

      return Promise.all([
        this._listRequests(false),
        subscription
      ]);
    }

    return Promise.all([
      this._listRequests(true),
      this._loggingSubscribe()
    ]);
  }

  updateSubscriptions (requests) {
    return this._updateSubscriptions('signer_requestsToConfirm', null, requests);
  }

  _listRequests = (doTimeout) => {
    const nextTimeout = (timeout = 1000, forceTimeout = doTimeout) => {
      if (forceTimeout) {
        setTimeout(() => {
          this._listRequests(doTimeout);
        }, timeout);
      }
    };

    if (!this._api.transport.isConnected) {
      nextTimeout(500, true);
      return;
    }

    return this._api.signer
      .requestsToConfirm()
      .then((requests) => {
        this.updateSubscriptions(requests);
        nextTimeout();
      })
      .catch(() => nextTimeout());
  }

  _postTransaction (data) {
    const request = {
      transaction: outTransaction(data.params[0]),
      requestId: data.json.result.result
    };

    this._updateSubscriptions('parity_postTransaction', null, request);
  }

  _loggingSubscribe () {
    return this._subscriber.subscribe('logging', (error, data) => {
      if (error || !data) {
        return;
      }

      switch (data.method) {
        case 'eth_sendTransaction':
        case 'eth_sendRawTransaction':
          this._listRequests(false);
          return;

        case 'parity_postTransaction':
          this._postTransaction(data);
          this._listRequests(false);
          return;
      }
    });
  }
}
