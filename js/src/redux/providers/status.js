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

import { statusBlockNumber, statusCollection, statusLogs } from './statusActions';
import { isEqual } from 'lodash';

export default class Status {
  constructor (store, api) {
    this._api = api;
    this._store = store;

    this._apiStatus = {};
    this._status = {};
    this._longStatus = {};
    this._minerSettings = {};

    this._timeoutIds = {};
    this._blockNumberSubscriptionId = null;

    this._timestamp = Date.now();

    api.transport.on('close', () => {
      this.stop();

      const apiStatus = this.getApiStatus();
      this._apiStatus = apiStatus;
      this._store.dispatch(statusCollection(apiStatus));
    });

    api.transport.on('open', () => {
      this.start();
    });
  }

  start () {
    this.stop();

    this._subscribeBlockNumber();
    this._pollLongStatus(true);
  }

  startPolling () {
    this._pollLogs();
    this._pollStatus();
  }

  stop () {
    if (this._blockNumberSubscriptionId) {
      this._api.unsubscribe(this._blockNumberSubscriptionId);
      this._blockNumberSubscriptionId = null;
    }

    Object.values(this._timeoutIds).forEach((timeoutId) => {
      clearTimeout(timeoutId);
    });
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

    const { isConnected } = this._api;
    const apiStatus = this.getApiStatus();

    const hasConnected = !this._apiStatus.isConnected && apiStatus.isConnected;

    if (hasConnected) {
      this._pollLongStatus(hasConnected);
    }

    if (!isEqual(apiStatus, this._apiStatus)) {
      this._store.dispatch(statusCollection(apiStatus));
      this._apiStatus = apiStatus;
    }

    if (!isConnected) {
      return nextTimeout(250);
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
        return nextTimeout();
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
  _pollLongStatus = (hasConnected = false) => {
    if (!this._api.isConnected) {
      return;
    }

    const nextTimeout = (timeout = 30000, hasConnected = false) => {
      if (this._timeoutIds.longStatus) {
        clearTimeout(this._timeoutIds.longStatus);
      }

      this._timeoutIds.longStatus = setTimeout(() => this._pollLongStatus(hasConnected), timeout);
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
        this._api.parity.enode()
      ])
      .then(([
        netPeers, clientVersion, netVersion, defaultExtraData, netChain, netPort, rpcSettings, enode
      ]) => {
        const isTest =
          netVersion === '2' || // morden
          netVersion === '3'; // ropsten

        const longStatus = {
          netPeers,
          clientVersion,
          defaultExtraData,
          netChain,
          netPort,
          rpcSettings,
          isTest,
          enode
        };

        if (!isEqual(longStatus, this._longStatus)) {
          this._store.dispatch(statusCollection(longStatus));
          this._longStatus = longStatus;
        }

        if (hasConnected) {
          this.startPolling();
        }

        return false;
      })
      .catch((error) => {
        // Try again soon if just got reconnected (network might take some time
        // to boot up)
        if (hasConnected) {
          nextTimeout(500, true);
          return true;
        }

        console.error('_pollLongStatus', error);
        return false;
      })
      .then((called) => {
        if (!called) {
          nextTimeout(60000);
        }
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
      return nextTimeout();
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
