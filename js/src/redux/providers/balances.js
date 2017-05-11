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

import { fetchBalances, fetchTokensBalances, queryTokensFilter } from './balancesActions';
import { loadTokens, fetchTokens } from './tokensActions';
import { padRight } from '~/api/util/format';

import Contracts from '~/contracts';

let instance = null;

export default class Balances {
  constructor (store, api) {
    this._api = api;
    this._store = store;

    this._tokenreg = null;
    this._tokenregSID = null;
    this._tokenMetaSID = null;

    this._blockNumberSID = null;
    this._accountsInfoSID = null;

    // Throtthled load tokens (no more than once
    // every minute)
    this.loadTokens = throttle(
      this._loadTokens,
      60 * 1000,
      { leading: true, trailing: true }
    );

    // Throttled `_fetchBalances` function
    // that gets called max once every 40s
    this.longThrottledFetch = throttle(
      this._fetchBalances,
      40 * 1000,
      { leading: false, trailing: true }
    );

    this.shortThrottledFetch = throttle(
      this._fetchBalances,
      2 * 1000,
      { leading: false, trailing: true }
    );

    // Fetch all tokens every 2 minutes
    this.throttledTokensFetch = throttle(
      this._fetchTokens,
      2 * 60 * 1000,
      { leading: false, trailing: true }
    );

    // Unsubscribe previous instance if it exists
    if (instance) {
      Balances.stop();
    }
  }

  static get (store = {}) {
    if (!instance && store) {
      const { api } = store.getState();

      return Balances.instantiate(store, api);
    }

    return instance;
  }

  static instantiate (store, api) {
    if (!instance) {
      instance = new Balances(store, api);
    }

    return instance;
  }

  static start () {
    if (!instance) {
      return Promise.reject('BalancesProvider has not been intiated yet');
    }

    const self = instance;

    // Unsubscribe from previous subscriptions
    return Balances
      .stop()
      .then(() => self.loadTokens())
      .then(() => {
        const promises = [
          self.subscribeBlockNumber(),
          self.subscribeAccountsInfo()
        ];

        return Promise.all(promises);
      });
  }

  static stop () {
    if (!instance) {
      return Promise.resolve();
    }

    const self = instance;
    const promises = [];

    if (self._blockNumberSID) {
      const p = self._api
        .unsubscribe(self._blockNumberSID)
        .then(() => {
          self._blockNumberSID = null;
        });

      promises.push(p);
    }

    if (self._accountsInfoSID) {
      const p = self._api
        .unsubscribe(self._accountsInfoSID)
        .then(() => {
          self._accountsInfoSID = null;
        });

      promises.push(p);
    }

    // Unsubscribe without adding the promises
    // to the result, since it would have to wait for a
    // reconnection to resolve if the Node is disconnected
    if (self._tokenreg) {
      if (self._tokenregSID) {
        const tokenregSID = self._tokenregSID;

        self._tokenreg
          .unsubscribe(tokenregSID)
          .then(() => {
            if (self._tokenregSID === tokenregSID) {
              self._tokenregSID = null;
            }
          });
      }

      if (self._tokenMetaSID) {
        const tokenMetaSID = self._tokenMetaSID;

        self._tokenreg
          .unsubscribe(tokenMetaSID)
          .then(() => {
            if (self._tokenMetaSID === tokenMetaSID) {
              self._tokenMetaSID = null;
            }
          });
      }
    }

    return Promise.all(promises);
  }

  subscribeAccountsInfo () {
    return this._api
      .subscribe('parity_allAccountsInfo', (error, accountsInfo) => {
        if (error) {
          return;
        }

        this.fetchAllBalances();
      })
      .then((accountsInfoSID) => {
        this._accountsInfoSID = accountsInfoSID;
      })
      .catch((error) => {
        console.warn('_subscribeAccountsInfo', error);
      });
  }

  subscribeBlockNumber () {
    return this._api
      .subscribe('eth_blockNumber', (error) => {
        if (error) {
          return console.warn('_subscribeBlockNumber', error);
        }

        this._store.dispatch(queryTokensFilter());
        return this.fetchAllBalances();
      })
      .then((blockNumberSID) => {
        this._blockNumberSID = blockNumberSID;
      })
      .catch((error) => {
        console.warn('_subscribeBlockNumber', error);
      });
  }

  fetchAllBalances (options = {}) {
    // If it's a network change, reload the tokens
    // ( and then fetch the tokens balances ) and fetch
    // the accounts balances
    if (options.changedNetwork) {
      this.loadTokens({ skipNotifications: true });
      this.loadTokens.flush();

      this.fetchBalances({
        force: true,
        skipNotifications: true
      });

      return;
    }

    this.fetchTokensBalances(options);
    this.fetchBalances(options);
  }

  fetchTokensBalances (options) {
    const { skipNotifications = false, force = false } = options;

    this.throttledTokensFetch(skipNotifications);

    if (force) {
      this.throttledTokensFetch.flush();
    }
  }

  fetchBalances (options) {
    const { skipNotifications = false, force = false } = options;
    const { syncing } = this._store.getState().nodeStatus;

    // If syncing, only retrieve balances once every
    // few seconds
    if (syncing || syncing === null) {
      this.shortThrottledFetch.cancel();
      this.longThrottledFetch(skipNotifications);

      if (force) {
        this.longThrottledFetch.flush();
      }

      return;
    }

    this.longThrottledFetch.cancel();
    this.shortThrottledFetch(skipNotifications);

    if (force) {
      this.shortThrottledFetch.flush();
    }
  }

  _fetchBalances (skipNotifications = false) {
    this._store.dispatch(fetchBalances(null, skipNotifications));
  }

  _fetchTokens (skipNotifications = false) {
    this._store.dispatch(fetchTokensBalances(null, null, skipNotifications));
  }

  getTokenRegistry () {
    return Contracts.get().tokenReg.getContract();
  }

  _loadTokens (options = {}) {
    return this
      .getTokenRegistry()
      .then((tokenreg) => {
        this._tokenreg = tokenreg;

        this._store.dispatch(loadTokens(options));

        return this.attachToTokens(tokenreg);
      })
      .catch((error) => {
        console.warn('balances::loadTokens', error);
      });
  }

  attachToTokens (tokenreg) {
    return Promise
      .all([
        this.attachToTokenMetaChange(tokenreg),
        this.attachToNewToken(tokenreg)
      ]);
  }

  attachToNewToken (tokenreg) {
    if (this._tokenregSID) {
      return Promise.resolve();
    }

    return tokenreg.instance.Registered
      .subscribe({
        fromBlock: 0,
        toBlock: 'latest',
        skipInitFetch: true
      }, (error, logs) => {
        if (error) {
          return console.error('balances::attachToNewToken', 'failed to attach to tokenreg Registered', error.toString(), error.stack);
        }

        this.handleTokensLogs(logs);
      })
      .then((tokenregSID) => {
        this._tokenregSID = tokenregSID;
      });
  }

  attachToTokenMetaChange (tokenreg) {
    if (this._tokenMetaSID) {
      return Promise.resolve();
    }

    return tokenreg.instance.MetaChanged
      .subscribe({
        fromBlock: 0,
        toBlock: 'latest',
        topics: [ null, padRight(this._api.util.asciiToHex('IMG'), 32) ],
        skipInitFetch: true
      }, (error, logs) => {
        if (error) {
          return console.error('balances::attachToTokenMetaChange', 'failed to attach to tokenreg MetaChanged', error.toString(), error.stack);
        }

        this.handleTokensLogs(logs);
      })
      .then((tokenMetaSID) => {
        this._tokenMetaSID = tokenMetaSID;
      });
  }

  handleTokensLogs (logs) {
    const tokenIds = logs.map((log) => log.params.id.value.toNumber());

    this._store.dispatch(fetchTokens(tokenIds));
  }
}
