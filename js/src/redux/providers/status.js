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
import { statusBlockNumber, statusCollection, statusLogs } from './statusActions';

const log = getLogger(LOG_KEYS.Signer);
let instance = null;

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

  start () {
    log.debug('status::start');

    Promise
      .all([
        this._subscribeBlockNumber(),

        this._pollLogs(),
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
        if (error) {
          return;
        }

        this._store.dispatch(statusBlockNumber(blockNumber));

        this._api.eth
          .getBlockByNumber(blockNumber)
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
            console.warn('status._subscribeBlockNumber', 'getBlockByNumber', error);
          });
      })
      .then((blockNumberSubscriptionId) => {
        this._blockNumberSubscriptionId = blockNumberSubscriptionId;
      });
  }

  _pollTraceMode = () => {
    return this._api.trace.block()
      .then(blockTraces => {
        // Assumes not in Trace Mode if no transactions
        // in latest block...
        return blockTraces.length > 0;
      })
      .catch(() => false);
  }

  getApiStatus = () => {
    const { isConnected, isConnecting, needsToken, secureToken } = this._api;

    const apiStatus = {
      isConnected,
      isConnecting,
      needsToken,
      secureToken
    };

    return apiStatus;
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

    const { refreshStatus } = this._store.getState().nodeStatus;

    const statusPromises = [ this._api.eth.syncing() ];

    if (refreshStatus) {
      statusPromises.push(this._api.parity.netPeers());
      statusPromises.push(this._api.eth.hashrate());
    }

    return Promise
      .all(statusPromises)
      .then(([ syncing, ...statusResults ]) => {
        const status = statusResults.length === 0
          ? { syncing }
          : {
            syncing,
            netPeers: statusResults[0],
            hashrate: statusResults[1]
          };

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

  /**
   * Miner settings should never changes unless
   * Parity is restarted, or if the values are changed
   * from the UI
   */
  _pollMinerSettings = () => {
    return Promise
      .all([
        this._api.eth.coinbase(),
        this._api.parity.extraData(),
        this._api.parity.minGasPrice(),
        this._api.parity.gasFloorTarget()
      ])
      .then(([
        coinbase, extraData, minGasPrice, gasFloorTarget
      ]) => {
        const minerSettings = {
          coinbase,
          extraData,
          minGasPrice,
          gasFloorTarget
        };

        if (!isEqual(minerSettings, this._minerSettings)) {
          this._store.dispatch(statusCollection(minerSettings));
          this._minerSettings = minerSettings;
        }
      })
      .catch((error) => {
        console.error('_pollMinerSettings', error);
      });
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

    const nextTimeout = (timeout = 30000) => {
      if (this._timeoutIds.longStatus) {
        clearTimeout(this._timeoutIds.longStatus);
      }

      this._timeoutIds.longStatus = setTimeout(() => this._pollLongStatus(), timeout);
    };

    // Poll Miner settings just in case
    const minerPromise = this._pollMinerSettings();

    const mainPromise = Promise
      .all([
        this._api.parity.netPeers(),
        this._api.web3.clientVersion(),
        this._api.net.version(),
        this._api.parity.defaultExtraData(),
        this._api.parity.netChain(),
        this._api.parity.netPort(),
        this._api.parity.rpcSettings(),
        this._api.parity.enode(),
        this._upgradeStore.checkUpgrade()
      ])
      .then(([
        netPeers, clientVersion, netVersion, defaultExtraData, netChain, netPort, rpcSettings, enode, upgradeStatus
      ]) => {
        const isTest = [
          '2', // morden
          '3', // ropsten
          '42' // kovan
        ].includes(netVersion);

        const longStatus = {
          netPeers,
          clientVersion,
          defaultExtraData,
          netChain,
          netPort,
          netVersion,
          rpcSettings,
          isTest,
          enode
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

    return Promise.all([ minerPromise, mainPromise ]);
  }

  _pollLogs = () => {
    const nextTimeout = (timeout = 1000) => {
      if (this._timeoutIds.logs) {
        clearTimeout(this._timeoutIds.logs);
      }

      this._timeoutIds.logs = setTimeout(this._pollLogs, timeout);
    };

    const { devLogsEnabled } = this._store.getState().nodeStatus;

    if (!devLogsEnabled) {
      nextTimeout();
      return Promise.resolve();
    }

    return Promise
      .all([
        this._api.parity.devLogs(),
        this._api.parity.devLogsLevels()
      ])
      .then(([devLogs, devLogsLevels]) => {
        this._store.dispatch(statusLogs({
          devLogs: devLogs.slice(-1024),
          devLogsLevels
        }));
      })
      .catch((error) => {
        console.error('_pollLogs', error);
      })
      .then(() => {
        return nextTimeout();
      });
  }
}
