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

    this.__pingable = false;
    this._apiStatus = {};
    this._status = {};
  }

  start () {
    this._subscribeBlockNumber();
    this._pollPing();
    this._pollStatus();
    this._pollLogs();
    this._fetchEnode();
  }

  _fetchEnode () {
    this._api.parity
      .enode()
      .then((enode) => {
        if (this._store.getState().nodeStatus.enode !== enode) {
          this._store.dispatch(statusCollection({ enode }));
        }
      })
      .catch(() => {
        window.setTimeout(() => {
          this._fetchEnode();
        }, 1000);
      });
  }

  _subscribeBlockNumber () {
    this._api
      .subscribe('eth_blockNumber', (error, blockNumber) => {
        if (error) {
          return;
        }

        this._store.dispatch(statusBlockNumber(blockNumber));
      });
  }

  _pollPing = () => {
    const dispatch = (status, timeout = 500) => {
      if (status !== this._pingable) {
        this._pingable = status;
        this._store.dispatch(statusCollection({ isPingable: status }));
      }

      setTimeout(this._pollPing, timeout);
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
    const { secureToken, isConnected, isConnecting, needsToken } = this._api;

    const nextTimeout = (timeout = 1000) => {
      setTimeout(this._pollStatus, timeout);
    };

    const wasConnected = this._store.getState().nodeStatus.isConnected;
    if (isConnected && !wasConnected) {
      this._fetchEnode();
      this._pollTraceMode()
        .then((traceMode) => {
          this._store.dispatch(statusCollection({ traceMode }));
        });
    }

    const apiStatus = {
      isConnected,
      isConnecting,
      needsToken,
      secureToken
    };

    if (!isEqual(apiStatus, this._apiStatus)) {
      this._store.dispatch(statusCollection(apiStatus));
      this._apiStatus = apiStatus;
    }

    if (!isConnected) {
      nextTimeout(250);
      return;
    }

    Promise
      .all([
        this._api.web3.clientVersion(),
        this._api.eth.coinbase(),
        this._api.parity.defaultExtraData(),
        this._api.parity.extraData(),
        this._api.parity.gasFloorTarget(),
        this._api.eth.hashrate(),
        this._api.parity.minGasPrice(),
        this._api.parity.netChain(),
        this._api.parity.netPeers(),
        this._api.parity.netPort(),
        this._api.parity.nodeName(),
        this._api.parity.rpcSettings(),
        this._api.eth.syncing()
      ])
      .then(([clientVersion, coinbase, defaultExtraData, extraData, gasFloorTarget, hashrate, minGasPrice, netChain, netPeers, netPort, nodeName, rpcSettings, syncing]) => {
        const isTest = netChain === 'morden' || netChain === 'testnet';

        const status = {
          clientVersion,
          coinbase,
          defaultExtraData,
          extraData,
          gasFloorTarget,
          hashrate,
          minGasPrice,
          netChain,
          netPeers,
          netPort,
          nodeName,
          rpcSettings,
          syncing,
          isTest
        };

        if (!isEqual(status, this._status)) {
          this._store.dispatch(statusCollection(status));
          this._status = status;
        }

        nextTimeout();
      })
      .catch((error) => {
        console.error('_pollStatus', error);
        nextTimeout();
      });
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
