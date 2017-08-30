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
import { action, observable } from 'mobx';

export default class StatusStore {
  @observable defaultExtraData = '';
  @observable enode = '';
  @observable blockNumber = new BigNumber(0);
  @observable blockTimestamp = new Date();
  @observable chain = '';
  @observable netPeers = new BigNumber(0);
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
    this.api.transport.on('close', () => {
      if (this.isPolling) {
        this.startPolling();
      }
    });
  }

  @action setStatuses ({ chain, defaultExtraData, enode, netPeers, netPort, rpcSettings, hashrate }) {
    this.chain = chain;
    this.defaultExtraData = defaultExtraData;
    this.enode = enode;
    this.netPeers = netPeers;
    this.netPort = netPort;
    this.rpcSettings = rpcSettings;
    this.hashrate = hashrate;
  }

  @action setMinerSettings ({ coinbase, extraData, gasFloorTarget, minGasPrice }) {
    this.coinbase = coinbase;
    this.extraData = extraData;
    this.gasFloorTarget = gasFloorTarget;
    this.minGasPrice = minGasPrice;
  }

  stopPolling () {
    this.isPolling = false;
    this.subscription.then(id => this.api.pubsub.unsubscribe([id]));
  }

  startPolling () {
    this.isPolling = true;
    this.subscription = this.api.pubsub.parity.getBlockHeaderByNumber((error, block) => {
      if (error) {
        console.warn('_startPolling', error);
        return;
      }
      this.subscribed = true;
      this.blockNumber = block.number;
      this.blockTimestamp = block.timestamp;
      this._pollMinerSettings();
      Promise
        .all([
          this.api.parity.chain(),
          this.api.parity.defaultExtraData(),
          this.api.parity.enode().then((enode) => enode).catch(() => '-'),
          this.api.parity.netPeers(),
          this.api.parity.netPort(),
          this.api.parity.rpcSettings(),
          this.api.eth.hashrate()
        ])
        .then(([
          chain, defaultExtraData, enode, netPeers, netPort, rpcSettings, hashrate
        ]) => {
          this.setStatuses({
            chain, defaultExtraData, enode, netPeers, netPort, rpcSettings, hashrate
          });
        })
        .catch((error) => {
          console.error('_pollStatuses', error);
          return;
        });
    });
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

  handleUpdateSetting = () => {
    return this._pollMinerSettings();
  };
}
