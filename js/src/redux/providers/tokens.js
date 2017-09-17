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

import { updateTokensFilter } from './balancesActions';
import { loadTokens, fetchTokens } from './tokensActions';
import { padRight } from '~/api/util/format';

import Contracts from '~/contracts';

let instance = null;

export default class Tokens {
  constructor (store, api) {
    this._api = api;
    this._store = store;

    this._tokenreg = null;
    this._tokenregSubs = [];

    this._loading = false;
  }

  get loading () {
    return this._loading;
  }

  static get (store) {
    if (!instance && store) {
      return Tokens.init(store);
    } else if (!instance) {
      throw new Error('The Tokens Provider has not been initialized yet');
    }

    return instance;
  }

  static init (store) {
    const { api } = store.getState();

    if (!instance) {
      instance = new Tokens(store, api);
    }

    return instance;
  }

  static start () {
    if (!instance) {
      return Promise.reject('Tokens Provider has not been initiated yet');
    }

    const self = instance;

    self._loading = true;

    // Unsubscribe from previous subscriptions
    return Tokens.stop()
      .then(() => self.loadTokens())
      .then(() => {
        self._loading = false;
      });
  }

  static stop () {
    if (!instance) {
      return Promise.resolve();
    }

    const self = instance;

    // Unsubscribe without adding the promises
    // to the result, since it would have to wait for a
    // reconnection to resolve if the Node is disconnected
    if (self._tokenreg) {
      const tokenregPromises = self._tokenregSubs
        .map((tokenregSID) => self._tokenreg.unsubscribe(tokenregSID));

      Promise.all(tokenregPromises)
        .then(() => {
          self._tokenregSubs = [];
        });
    }

    return Promise.resolve();
  }

  attachToTokensEvents (tokenreg) {
    const metaTopics = [ null, padRight(this._api.util.asciiToHex('IMG'), 32) ];

    return Promise
      .all([
        this._attachToTokenregEvents(tokenreg, 'Registered'),
        this._attachToTokenregEvents(tokenreg, 'MetaChanged', metaTopics)
      ]);
  }

  getTokenRegistry () {
    return Contracts.get().tokenReg.getContract();
  }

  loadTokens (options = {}) {
    const { dispatch, getState } = this._store;

    return this
      .getTokenRegistry()
      .then((tokenreg) => {
        this._tokenreg = tokenreg;

        return loadTokens(options)(dispatch, getState);
      })
      .then(() => updateTokensFilter()(dispatch, getState))
      .then(() => this.attachToTokensEvents(this._tokenreg))
      .catch((error) => {
        console.warn('balances::loadTokens', error);
      });
  }

  _attachToTokenregEvents (tokenreg, event, topics = []) {
    if (this._tokenregSID) {
      return Promise.resolve();
    }

    return tokenreg.instance[event]
      .subscribe({
        fromBlock: 'latest',
        toBlock: 'latest',
        topics: topics,
        skipInitFetch: true
      }, (error, logs) => {
        if (error) {
          return console.error('balances::attachToNewToken', 'failed to attach to tokenreg Registered', error.toString(), error.stack);
        }

        this._handleTokensLogs(logs);
      })
      .then((tokenregSID) => {
        this._tokenregSubs.push(tokenregSID);
      });
  }

  _handleTokensLogs (logs) {
    const { dispatch, getState } = this._store;
    const tokenIds = logs.map((log) => log.params.id.value.toNumber());

    return fetchTokens(tokenIds)(dispatch, getState)
      .then(() => updateTokensFilter()(dispatch, getState));
  }
}
