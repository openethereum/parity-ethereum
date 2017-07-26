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

import BigNumber from 'bignumber.js';
import { action, computed, observable, transaction } from 'mobx';

import { NetChain } from '@parity/ui';

console.log('NetChain', NetChain, NetChain.Store);

export default class StatusStore {
  @observable defaultExtraData = '';
  @observable enode = '';
  @observable hashrate = new BigNumber(0);
  @observable netPort = new BigNumber(0);
  @observable nodeName = '';
  @observable rpcSettings = {};

  @observable coinbase = '';
  @observable extraData = '';
  @observable gasFloorTarget = new BigNumber(0);
  @observable minGasPrice = new BigNumber(0);

  api = null;
  _timeoutIds = {};

  constructor (api) {
    this.api = api;
    this.chainStore = NetChain.Store.get(api);

    this.startPolling();
  }

  @computed get netChain () {
    return this.chainStore.netChain;
  }

  @action setLongStatus ({ defaultExtraData, enode, netPort, rpcSettings }) {
    transaction(() => {
      this.defaultExtraData = defaultExtraData;
      this.enode = enode;
      this.netPort = netPort;
      this.rpcSettings = rpcSettings;
    });
  }

  @action setStatus ({ hashrate }) {
    transaction(() => {
      this.hashrate = hashrate;
    });
  }

  @action setMinerSettings ({ coinbase, extraData, gasFloorTarget, minGasPrice }) {
    transaction(() => {
      this.coinbase = coinbase;
      this.extraData = extraData;
      this.gasFloorTarget = gasFloorTarget;
      this.minGasPrice = minGasPrice;
    });
  }

  startPolling () {
    this._pollStatus();
    this._pollLongStatus();
  }

  stopPolling () {
    Object.keys(this._timeoutIds).forEach((key) => clearTimeout(this._timeoutIds[key]));
  }

  /**
   * Miner settings should never changes unless
   * Parity is restarted, or if the values are changed
   * from the UI
   */
  _pollMinerSettings () {
    return Promise
      .all([
        this.api.eth.coinbase(),
        this.api.parity.extraData(),
        this.api.parity.gasFloorTarget(),
        this.api.parity.minGasPrice()
      ])
      .then(([
        coinbase, extraData, gasFloorTarget, minGasPrice
      ]) => {
        const minerSettings = {
          coinbase,
          extraData,
          gasFloorTarget,
          minGasPrice
        };

        this.setMinerSettings(minerSettings);
      })
      .catch((error) => {
        console.error('_pollMinerSettings', error);
      });
  }

  _pollStatus () {
    const nextTimeout = (timeout = 1000) => {
      clearTimeout(this._timeoutIds.short);
      this._timeoutIds.short = setTimeout(() => this._pollStatus(), timeout);
    };

    return Promise
      .all([
        this.api.eth.hashrate()
      ])
      .then(([
        hashrate
      ]) => {
        this.setStatus({
          hashrate
        });
      })
      .catch((error) => {
        console.error('_pollStatus', error);
      })
      .then(() => {
        nextTimeout();
      });
  }

  _pollLongStatus () {
    const nextTimeout = (timeout = 30000) => {
      clearTimeout(this._timeoutIds.long);
      this._timeoutIds.long = setTimeout(() => this._pollLongStatus(), timeout);
    };

    this._pollMinerSettings();
    return Promise
      .all([
        this.api.parity.defaultExtraData(),
        this.api.parity.enode().then((enode) => enode).catch(() => '-'),
        this.api.parity.netPort(),
        this.api.parity.rpcSettings()
      ])
      .then(([
        defaultExtraData, enode, netPort, rpcSettings
      ]) => {
        this.setLongStatus({
          defaultExtraData, enode, netPort, rpcSettings
        });
      })
      .catch((error) => {
        console.error('_pollLongStatus', error);
      })
      .then(() => {
        nextTimeout();
      });
  }

  handleUpdateSetting = () => {
    return this._pollMinerSettings();
  };
}
