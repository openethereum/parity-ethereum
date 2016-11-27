// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

    this._pingable = false;
    this._apiStatus = {};
    this._status = {};
    this._longStatus = {};
    this._minerSettings = {};

    this._pollPingTimeoutId = null;
    this._longStatusTimeoutId = null;

    this._timestamp = Date.now();
  }

  start () {
    this._subscribeBlockNumber();
    this._pollPing();
    this._pollStatus();
    this._pollLongStatus();
    this._pollLogs();
  }

  _subscribeBlockNumber () {
    this._api
      .subscribe('eth_blockNumber', (error, blockNumber) => {
        if (error) {
          return;
        }

        this._store.dispatch(statusBlockNumber(blockNumber));

        this._api.eth
          .getBlockByNumber(blockNumber)
          .then((block) => {
            this._store.dispatch(statusCollection({ gasLimit: block.gasLimit }));
          })
          .catch((error) => {
            console.warn('status._subscribeBlockNumber', 'getBlockByNumber', error);
          });
      })
      .then((subscriptionId) => {
        console.log('status._subscribeBlockNumber', 'subscriptionId', subscriptionId);
      });
  }

  /**
   * Pinging should be smart. It should only
   * be used when the UI is connecting or the
   * Node is deconnected.
   *
   * @see src/views/Connection/connection.js
   */
  _shouldPing = () => {
    const { isConnected } = this._apiStatus;
    return !isConnected;
  }

  _stopPollPing = () => {
    if (!this._pollPingTimeoutId) {
      return;
    }

    clearTimeout(this._pollPingTimeoutId);
    this._pollPingTimeoutId = null;
  }

  _pollPing = () => {
    // Already pinging, don't try again
    if (this._pollPingTimeoutId) {
      return;
    }

    const dispatch = (pingable, timeout = 1000) => {
      if (pingable !== this._pingable) {
        this._pingable = pingable;
        this._store.dispatch(statusCollection({ isPingable: pingable }));
      }

      this._pollPingTimeoutId = setTimeout(() => {
        this._stopPollPing();
        this._pollPing();
      }, timeout);
    };

    fetch('/', { method: 'HEAD' })
      .then((response) => dispatch(!!response.ok))
      .catch(() => dispatch(false));
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

  _pollStatus = () => {
    const nextTimeout = (timeout = 1000) => {
      setTimeout(() => this._pollStatus(), timeout);
    };

    const { isConnected, isConnecting, needsToken, secureToken } = this._api;

    const apiStatus = {
      isConnected,
      isConnecting,
      needsToken,
      secureToken
    };

    const gotConnected = !this._apiStatus.isConnected && apiStatus.isConnected;

    if (gotConnected) {
      this._pollLongStatus();
      this._store.dispatch(statusCollection({ isPingable: true }));
    }

    if (!isEqual(apiStatus, this._apiStatus)) {
      this._store.dispatch(statusCollection(apiStatus));
      this._apiStatus = apiStatus;
    }

    // Ping if necessary, otherwise stop pinging
    if (this._shouldPing()) {
      this._pollPing();
    } else {
      this._stopPollPing();
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

    Promise
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
      });

    nextTimeout();
  }

  /**
   * Miner settings should never changes unless
   * Parity is restarted, or if the values are changed
   * from the UI
   */
  _pollMinerSettings = () => {
    Promise
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
      return;
    }

    const nextTimeout = (timeout = 30000) => {
      if (this._longStatusTimeoutId) {
        clearTimeout(this._longStatusTimeoutId);
      }

      this._longStatusTimeoutId = setTimeout(this._pollLongStatus, timeout);
    };

    // Poll Miner settings just in case
    this._pollMinerSettings();

    Promise
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
      })
      .catch((error) => {
        console.error('_pollLongStatus', error);
      });

    nextTimeout(60000);
  }

  _pollLogs = () => {
    const nextTimeout = (timeout = 1000) => setTimeout(this._pollLogs, timeout);
    const { devLogsEnabled } = this._store.getState().nodeStatus;

    if (!devLogsEnabled) {
      nextTimeout();
      return;
    }

    Promise
      .all([
        this._api.parity.devLogs(),
        this._api.parity.devLogsLevels()
      ])
      .then(([devLogs, devLogsLevels]) => {
        this._store.dispatch(statusLogs({
          devLogs: devLogs.slice(-1024),
          devLogsLevels
        }));
        nextTimeout();
      })
      .catch((error) => {
        console.error('_pollLogs', error);
        nextTimeout();
      });
  }
}
