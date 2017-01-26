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

export default class Signer {
  constructor (updateSubscriptions, api, subscriber) {
    this._subscriber = subscriber;
    this._api = api;
    this._updateSubscriptions = updateSubscriptions;
    this._started = false;
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    return Promise.all([
      this._listRequests(true),
      this._loggingSubscribe()
    ]);
  }

  _listRequests = (doTimeout) => {
    const nextTimeout = (timeout = 1000) => {
      if (doTimeout) {
        setTimeout(() => {
          this._listRequests(true);
        }, timeout);
      }
    };

    if (!this._api.transport.isConnected) {
      nextTimeout(500);
      return;
    }

    return this._api.signer
      .requestsToConfirm()
      .then((requests) => {
        this._updateSubscriptions('signer_requestsToConfirm', null, requests);
        nextTimeout();
      })
      .catch(nextTimeout);
  }

  _loggingSubscribe () {
    return this._subscriber.subscribe('logging', (error, data) => {
      if (error || !data) {
        return;
      }

      switch (data.method) {
        case 'parity_postTransaction':
        case 'eth_sendTranasction':
        case 'eth_sendRawTransaction':
          this._listRequests(false);
          return;
      }
    });
  }
}
