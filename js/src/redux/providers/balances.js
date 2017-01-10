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

import { throttle } from 'lodash';

import { loadTokens, setTokenReg, fetchBalances, fetchTokens, fetchTokensBalances } from './balancesActions';
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

    // Throttled `retrieveTokens` function
    // that gets called max once every 40s
    this.longThrottledFetch = throttle(
      this.fetchBalances,
      40 * 1000,
      { leading: true }
    );

    this.shortThrottledFetch = throttle(
      this.fetchBalances,
      2 * 1000,
      { leading: true }
    );

    // Fetch all tokens every 2 minutes
    this.throttledTokensFetch = throttle(
      this.fetchTokens,
      60 * 1000,
      { leading: true }
    );

    // Unsubscribe previous instance if it exists
    if (instance) {
      Balances.stop();
    }

    instance = this;
  }

  static instantiate (store, api) {
    return new Balances(store, api);
  }

  static start () {
    if (!instance) {
      return Promise.reject('BalancesProvider has not been intiated yet');
    }

    const self = instance;

    // Unsubscribe from previous subscriptions
    return Balances
      .stop()
      .then(() => {
        const promises = [
          self.subscribeBlockNumber(),
          self.subscribeAccountsInfo(),

          self.loadTokens()
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

    if (self._tokenreg) {
      if (self._tokenregSID) {
        const p = self._tokenreg
          .unsubscribe(self._tokenregSID)
          .then(() => {
            self._tokenregSID = null;
          });

        promises.push(p);
      }

      if (self._tokenMetaSID) {
        const p = self._tokenreg
          .unsubscribe(self._tokenMetaSID)
          .then(() => {
            self._tokenMetaSID = null;
          });

        promises.push(p);
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

        return this.fetchAllBalances();
      })
      .then((blockNumberSID) => {
        this._blockNumberSID = blockNumberSID;
      })
      .catch((error) => {
        console.warn('_subscribeBlockNumber', error);
      });
  }

  fetchAllBalances () {
    const { syncing } = this._store.getState().nodeStatus;

    this.throttledTokensFetch();

    // If syncing, only retrieve balances once every
    // few seconds
    if (syncing) {
      this.shortThrottledFetch.cancel();
      return this.longThrottledFetch();
    }

    this.longThrottledFetch.cancel();
    return this.shortThrottledFetch();
  }

  fetchBalances () {
    this._store.dispatch(fetchBalances());
  }

  fetchTokens () {
    this._store.dispatch(fetchTokensBalances());
  }

  getTokenRegistry () {
    return Contracts.get().tokenReg.getContract();
  }

  loadTokens () {
    return this
      .getTokenRegistry()
      .then((tokenreg) => {
        this._tokenreg = tokenreg;

        this._store.dispatch(setTokenReg(tokenreg));
        this._store.dispatch(loadTokens());

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
