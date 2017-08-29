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

import { isEqual } from 'lodash';

import { LOG_KEYS, getLogger } from '~/config';
import UpgradeStore from '~/modals/UpgradeParity/store';

import BalancesProvider from './balances';
import { statusBlockNumber, statusCollection } from './statusActions';

const log = getLogger(LOG_KEYS.Signer);
let instance = null;

const STATUS_OK = 'ok';
const STATUS_WARN = 'needsAttention';
const STATUS_BAD = 'bad';

export default class Status {
  _apiStatus = {};
  _status = {};
  _longStatus = {};
  _minerSettings = {};
  _timeoutIds = {};
  _blockNumberSubscriptionId = null;
  _timestamp = Date.now();

  constructor (store, api) {
    this._api = api;
    this._store = store;
    this._upgradeStore = UpgradeStore.get(api);

    // On connecting, stop all subscriptions
    api.on('connecting', this.stop, this);

    // On connected, start the subscriptions
    api.on('connected', this.start, this);

    // On disconnected, stop all subscriptions
    api.on('disconnected', this.stop, this);

    this.updateApiStatus();
  }

  static instantiate (store, api) {
    if (!instance) {
      instance = new Status(store, api);
    }

    return instance;
  }

  static get () {
    if (!instance) {
      throw new Error('The Status Provider has not been initialized yet');
    }

    return instance;
  }

  start () {
    log.debug('status::start');

    Promise
      .all([
        this._subscribeBlockNumber(),

        this._pollLongStatus(),
        this._pollStatus()
      ])
      .then(() => {
        return BalancesProvider.start();
      });
  }

  stop () {
    log.debug('status::stop');

    const promises = [];

    if (this._blockNumberSubscriptionId) {
      const promise = this._api
        .unsubscribe(this._blockNumberSubscriptionId)
        .then(() => {
          this._blockNumberSubscriptionId = null;
        });

      promises.push(promise);
    }

    Object.values(this._timeoutIds).forEach((timeoutId) => {
      clearTimeout(timeoutId);
    });

    const promise = BalancesProvider.stop();

    promises.push(promise);

    return Promise.all(promises)
      .then(() => true)
      .catch((error) => {
        console.error('status::stop', error);
        return true;
      })
      .then(() => this.updateApiStatus());
  }

  updateApiStatus () {
    const apiStatus = this.getApiStatus();

    log.debug('status::updateApiStatus', apiStatus);

    if (!isEqual(apiStatus, this._apiStatus)) {
      this._store.dispatch(statusCollection(apiStatus));
      this._apiStatus = apiStatus;
    }
  }

  _subscribeBlockNumber = () => {
    return this._api
      .subscribe('eth_blockNumber', (error, blockNumber) => {
        if (error || !blockNumber) {
          return;
        }

        this._store.dispatch(statusBlockNumber(blockNumber));

        this._api.parity
          .getBlockHeaderByNumber(blockNumber)
          .then((block) => {
            if (!block) {
              return;
            }

            this._store.dispatch(statusCollection({
              blockTimestamp: block.timestamp,
              gasLimit: block.gasLimit
            }));
          })
          .catch((error) => {
            console.warn('status._subscribeBlockNumber', 'getBlockHeaderByNumber', error);
          });
      })
      .then((blockNumberSubscriptionId) => {
        this._blockNumberSubscriptionId = blockNumberSubscriptionId;
      });
  }

  _pollTraceMode = () => {
    return this._api.trace
      .block()
      .then(blockTraces => {
        // Assumes not in Trace Mode if no transactions
        // in latest block...
        return blockTraces.length > 0;
      })
      .catch(() => false);
  }

  getApiStatus = () => {
    const { isConnected, isConnecting, needsToken, secureToken } = this._api;

    return {
      isConnected,
      isConnecting,
      needsToken,
      secureToken
    };
  }

  _pollStatus = () => {
    const nextTimeout = (timeout = 1000) => {
      if (this._timeoutIds.status) {
        clearTimeout(this._timeoutIds.status);
      }

      this._timeoutIds.status = setTimeout(() => this._pollStatus(), timeout);
    };

    this.updateApiStatus();

    if (!this._api.isConnected) {
      nextTimeout(250);
      return Promise.resolve();
    }

    const statusPromises = [
      this._api.eth.syncing(),
      this._api.parity.netPeers(),
      this._api.parity.nodeHealth()
    ];

    return Promise
      .all(statusPromises)
      .then(([ syncing, netPeers, health ]) => {
        const status = { netPeers, syncing, health };

        health.overall = this._overallStatus(health);
        health.peers = health.peers || {};
        health.sync = health.sync || {};
        health.time = health.time || {};

        if (!isEqual(status, this._status)) {
          this._store.dispatch(statusCollection(status));
          this._status = status;
        }
      })
      .catch((error) => {
        console.error('_pollStatus', error);
      })
      .then(() => {
        nextTimeout();
      });
  }

  _overallStatus = (health) => {
    const allWithTime = [health.peers, health.sync, health.time].filter(x => x);
    const all = [health.peers, health.sync].filter(x => x);
    const statuses = all.map(x => x.status);
    const bad = statuses.find(x => x === STATUS_BAD);
    const needsAttention = statuses.find(x => x === STATUS_WARN);
    const message = allWithTime.map(x => x.message).filter(x => x);

    if (all.length) {
      return {
        status: bad || needsAttention || STATUS_OK,
        message
      };
    }

    return {
      status: STATUS_BAD,
      message: ['Unable to fetch node health.']
    };
  }

  /**
   * The data fetched here should not change
   * unless Parity is restarted. They are thus
   * fetched every 30s just in case, and whenever
   * the client got reconnected.
   */
  _pollLongStatus = () => {
    if (!this._api.isConnected) {
      return Promise.resolve();
    }

    const { nodeKindFull } = this._store.getState().nodeStatus;
    const defaultTimeout = (nodeKindFull === false ? 240 : 30) * 1000;

    const nextTimeout = (timeout = defaultTimeout) => {
      if (this._timeoutIds.longStatus) {
        clearTimeout(this._timeoutIds.longStatus);
      }

      this._timeoutIds.longStatus = setTimeout(() => this._pollLongStatus(), timeout);
    };

    const statusPromises = [
      this._api.parity.nodeKind(),
      this._api.parity.netPeers(),
      this._api.web3.clientVersion(),
      this._api.net.version(),
      this._api.parity.netChain()
    ];

    if (nodeKindFull) {
      statusPromises.push(this._upgradeStore.checkUpgrade());
    }

    return Promise
      .all(statusPromises)
      .then(([nodeKind, netPeers, clientVersion, netVersion, netChain]) => {
        const isTest = [
          '2',  // morden
          '3',  // ropsten,
          '17', // devchain
          '42'  // kovan
        ].includes(netVersion);

        const nodeKindFull = nodeKind &&
          nodeKind.availability === 'personal' &&
          nodeKind.capability === 'full';

        const longStatus = {
          nodeKind,
          nodeKindFull,
          netPeers,
          clientVersion,
          netChain,
          netVersion,
          isTest
        };

        if (!isEqual(longStatus, this._longStatus)) {
          this._store.dispatch(statusCollection(longStatus));
          this._longStatus = longStatus;
        }
      })
      .catch((error) => {
        console.error('_pollLongStatus', error);
      })
      .then(() => {
        nextTimeout(60000);
      });
  }
}
