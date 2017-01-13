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

export default class Balances {
  constructor (store, api) {
    this._api = api;
    this._store = store;

    this._tokenregSubId = null;
    this._tokenregMetaSubId = null;

    // Throttled `retrieveTokens` function
    // that gets called max once every 40s
    this.longThrottledFetch = throttle(
      this.fetchBalances,
      40 * 1000,
      { trailing: true }
    );

    this.shortThrottledFetch = throttle(
      this.fetchBalances,
      2 * 1000,
      { trailing: true }
    );

    // Fetch all tokens every 2 minutes
    this.throttledTokensFetch = throttle(
      this.fetchTokens,
      60 * 1000,
      { trailing: true }
    );
  }

  start () {
    this.subscribeBlockNumber();
    this.subscribeAccountsInfo();

    this.loadTokens();
  }

  subscribeAccountsInfo () {
    this._api
      .subscribe('parity_allAccountsInfo', (error, accountsInfo) => {
        if (error) {
          return;
        }

        this.fetchBalances();
      })
      .catch((error) => {
        console.warn('_subscribeAccountsInfo', error);
      });
  }

  subscribeBlockNumber () {
    this._api
      .subscribe('eth_blockNumber', (error) => {
        if (error) {
          return console.warn('_subscribeBlockNumber', error);
        }

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
      })
      .catch((error) => {
        console.warn('_subscribeBlockNumber', error);
      });
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
    this
      .getTokenRegistry()
      .then((tokenreg) => {
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
    if (this._tokenregSubId) {
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
      .then((tokenregSubId) => {
        this._tokenregSubId = tokenregSubId;
      });
  }

  attachToTokenMetaChange (tokenreg) {
    if (this._tokenregMetaSubId) {
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
      .then((tokenregMetaSubId) => {
        this._tokenregMetaSubId = tokenregMetaSubId;
      });
  }

  handleTokensLogs (logs) {
    const tokenIds = logs.map((log) => log.params.id.value.toNumber());
    this._store.dispatch(fetchTokens(tokenIds));
  }
}
