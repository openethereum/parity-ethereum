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

import { isEqual, debounce } from 'lodash';

import { LOG_KEYS, getLogger } from '~/config';
// import UpgradeStore from '~/modals/UpgradeParity/store';

import { statusBlockNumber, statusCollection } from './statusActions';

const log = getLogger(LOG_KEYS.Signer);
let instance = null;

const STATUS_OK = 'ok';
const STATUS_WARN = 'needsAttention';
const STATUS_BAD = 'bad';

export default class Status {
  _apiStatus = {};
  _longStatus = {};
  _minerSettings = {};
  _timeoutIds = {};
  _blockNumberSubscriptionId = null;
  _timestamp = Date.now();

  constructor (store, api) {
    this._api = api;
    this._store = store;
    // this._upgradeStore = UpgradeStore.get(api);

    this.updateApiStatus();
  }

  static init (store) {
    const { api } = store.getState();

    if (!instance) {
      instance = new Status(store, api);
    }

    return instance;
  }

  static get (store) {
    if (!instance && store) {
      return Status.init(store);
    } else if (!instance) {
      throw new Error('The Status Provider has not been initialized yet');
    }

    return instance;
  }

  static start () {
    const self = instance;

    log.debug('status::start');

    const promises = [
      self._subscribeBlockNumber(),
      self._subscribeNetPeers(),
      self._subscribeEthSyncing(),
      self._subscribeNodeHealth(),
      self._pollLongStatus(),
      self._pollApiStatus()
    ];

    return Status.stop()
      .then(() => Promise.all(promises));
  }

  static stop () {
    if (!instance) {
      return Promise.resolve();
    }

    const self = instance;

    log.debug('status::stop');

    self._clearTimeouts();

    return self._unsubscribeBlockNumber()
      .catch((error) => {
        console.error('status::stop', error);
      })
      .then(() => self.updateApiStatus());
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

  updateApiStatus () {
    const apiStatus = this.getApiStatus();

    log.debug('status::updateApiStatus', apiStatus);

    if (!isEqual(apiStatus, this._apiStatus)) {
      this._store.dispatch(statusCollection(apiStatus));
      this._apiStatus = apiStatus;
    }
  }

  _clearTimeouts () {
    Object.values(this._timeoutIds).forEach((timeoutId) => {
      clearTimeout(timeoutId);
    });
  }

  _overallStatus (health) {
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

  _updateStatus = debounce(status => {
    this._store.dispatch(statusCollection(status));
  }, 2500, {
    maxWait: 5000
  });

  _subscribeEthSyncing = () => {
    return this._api.pubsub
      .eth
      .syncing((error, syncing) => {
        if (error) {
          return;
        }

        this._updateStatus({ syncing });
      });
  }

  _subscribeNetPeers = () => {
    return this._api.pubsub
      .parity
      .netPeers((error, netPeers) => {
        if (error || !netPeers) {
          return;
        }

        this._store.dispatch(statusCollection({ netPeers }));
      });
  }

  _subscribeNodeHealth = () => {
    return this._api.pubsub
      .parity
      .nodeHealth((error, health) => {
        if (error || !health) {
          return;
        }

        health.overall = this._overallStatus(health);
        health.peers = health.peers || {};
        health.sync = health.sync || {};
        health.time = health.time || {};

        this._store.dispatch(statusCollection({ health }));
      });
  }

  _unsubscribeBlockNumber () {
    if (this._blockNumberSubscriptionId) {
      return this._api
        .unsubscribe(this._blockNumberSubscriptionId)
        .then(() => {
          this._blockNumberSubscriptionId = null;
        });
    }

    return Promise.resolve();
  }

  _pollApiStatus = () => {
    const nextTimeout = (timeout = 1000) => {
      if (this._timeoutIds.status) {
        clearTimeout(this._timeoutIds.status);
      }

      this._timeoutIds.status = setTimeout(() => this._pollApiStatus(), timeout);
    };

    this.updateApiStatus();

    if (!this._api.isConnected) {
      nextTimeout(250);
    } else {
      nextTimeout();
    }
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
    const defaultTimeout = (nodeKindFull === false ? 240 : 60) * 1000;

    const nextTimeout = (timeout = defaultTimeout) => {
      if (this._timeoutIds.longStatus) {
        clearTimeout(this._timeoutIds.longStatus);
      }

      this._timeoutIds.longStatus = setTimeout(() => this._pollLongStatus(), timeout);
    };

    const statusPromises = [
      this._api.parity.nodeKind(),
      this._api.web3.clientVersion(),
      this._api.net.version(),
      this._api.parity.netChain()
    ];

    // if (nodeKindFull) {
    //   statusPromises.push(this._upgradeStore.checkUpgrade());
    // }

    return Promise
      .all(statusPromises)
      .then(([nodeKind, clientVersion, netVersion, netChain]) => {
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
      .then(() => {
        nextTimeout();
      })
      .catch((error) => {
        console.error('_pollLongStatus', error);
        nextTimeout(30000);
      });
  }
}
