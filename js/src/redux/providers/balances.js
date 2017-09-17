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

import { throttle } from 'lodash';

import { LOG_KEYS, getLogger } from '~/config';

import { fetchBalances, queryTokensFilter, updateTokensFilter } from './balancesActions';

const log = getLogger(LOG_KEYS.Balances);
let instance = null;

export default class Balances {
  constructor (store, api) {
    this._api = api;
    this._store = store;

    this._apiSubs = [];

    // Throttled `_fetchEthBalances` function
    // that gets called max once every 40s
    this.longThrottledFetch = throttle(
      this._fetchEthBalances,
      40 * 1000,
      { leading: true, trailing: false }
    );

    this.shortThrottledFetch = throttle(
      this._fetchEthBalances,
      2 * 1000,
      { leading: true, trailing: false }
    );

    // Unsubscribe previous instance if it exists
    if (instance) {
      Balances.stop();
    }
  }

  static get (store) {
    if (!instance && store) {
      return Balances.init(store);
    } else if (!instance) {
      throw new Error('The Balances Provider has not been initialized yet');
    }

    return instance;
  }

  static init (store) {
    const { api } = store.getState();

    if (!instance) {
      instance = new Balances(store, api);
    }

    return instance;
  }

  static start () {
    if (!instance) {
      return Promise.reject('BalancesProvider has not been initiated yet');
    }

    const self = instance;

    // Unsubscribe from previous subscriptions
    return Balances.stop()
      .then(() => {
        const promises = [
          self.subscribeBlockNumber(),
          self.subscribeAccountsInfo()
        ];

        return Promise.all(promises);
      })
      .then(() => self.fetchEthBalances());
  }

  static stop () {
    if (!instance) {
      return Promise.resolve();
    }

    const self = instance;
    const promises = self._apiSubs.map((subId) => self._api.unsubscribe(subId));

    return Promise.all(promises)
      .then(() => {
        self._apiSubs = [];
      });
  }

  subscribeAccountsInfo () {
    // Don't trigger the balances updates on first call (when the
    // subscriptions are setup)
    let firstcall = true;

    return this._api
      .subscribe('parity_allAccountsInfo', (error, accountsInfo) => {
        if (error) {
          return console.warn('balances::subscribeAccountsInfo', error);
        }

        if (firstcall) {
          firstcall = false;
          return;
        }

        this._store.dispatch(updateTokensFilter());
        this.fetchEthBalances();
      })
      .then((subId) => {
        this._apiSubs.push(subId);
      })
      .catch((error) => {
        console.warn('_subscribeAccountsInfo', error);
      });
  }

  subscribeBlockNumber () {
    // Don't trigger the balances updates on first call (when the
    // subscriptions are setup)
    let firstcall = true;

    return this._api
      .subscribe('eth_blockNumber', (error, block) => {
        if (error) {
          return console.warn('balances::subscribeBlockNumber', error);
        }

        if (firstcall) {
          firstcall = false;
          return;
        }

        this._store.dispatch(queryTokensFilter());
        return this.fetchEthBalances();
      })
      .then((subId) => {
        this._apiSubs.push(subId);
      })
      .catch((error) => {
        console.warn('_subscribeBlockNumber', error);
      });
  }

  fetchEthBalances (options = {}) {
    log.debug('fetching eth balances (throttled)...');

    const { syncing } = this._store.getState().nodeStatus;

    if (options.force) {
      return this._fetchEthBalances();
    }

    // If syncing, only retrieve balances once every
    // few seconds
    if (syncing || syncing === null) {
      this.shortThrottledFetch.cancel();
      return this.longThrottledFetch();
    }

    this.longThrottledFetch.cancel();
    return this.shortThrottledFetch();
  }

  _fetchEthBalances (skipNotifications = false) {
    log.debug('fetching eth balances (real)...');

    const { dispatch, getState } = this._store;

    return fetchBalances(null, skipNotifications)(dispatch, getState);
  }
}
