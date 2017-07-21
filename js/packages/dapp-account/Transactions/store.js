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

import { action, observable, transaction } from 'mobx';

import etherscan from '@parity/etherscan';

export default class Store {
  @observable address = null;
  @observable isLoading = false;
  @observable isTracing = false;
  @observable netVersion = '0';
  @observable txHashes = [];

  constructor (api) {
    this._api = api;
  }

  @action setHashes = (transactions) => {
    transaction(() => {
      this.setLoading(false);
      this.txHashes = transactions.map((transaction) => transaction.hash);
    });
  }

  @action setAddress = (address) => {
    this.address = address;
  }

  @action setLoading = (isLoading) => {
    this.isLoading = isLoading;
  }

  @action setNetVersion = (netVersion) => {
    this.netVersion = netVersion;
  }

  @action setTracing = (isTracing) => {
    this.isTracing = isTracing;
  }

  @action updateProps = (props) => {
    transaction(() => {
      this.setAddress(props.address);
      this.setNetVersion(props.netVersion);

      // TODO: When tracing is enabled again, adjust to actually set
      this.setTracing(false && props.traceMode);
    });

    return this.getTransactions();
  }

  getTransactions () {
    if (this.netVersion === '0') {
      return Promise.resolve();
    }

    this.setLoading(true);

    // TODO: When supporting other chains (eg. ETC). call to be made to other endpoints
    return (
      this.isTracing
        ? this.fetchTraceTransactions()
        : this.fetchEtherscanTransactions()
      )
      .then((transactions) => {
        this.setHashes(transactions);
      })
      .catch((error) => {
        console.warn('getTransactions', error);
        this.setLoading(false);
      });
  }

  fetchEtherscanTransactions () {
    return etherscan.account.transactions(this.address, 0, false, this.netVersion);
  }

  fetchTraceTransactions () {
    return Promise
      .all([
        this._api.trace.filter({
          fromAddress: this.address,
          fromBlock: 0
        }),
        this._api.trace.filter({
          fromBlock: 0,
          toAddress: this.address
        })
      ])
      .then(([fromTransactions, toTransactions]) => {
        return fromTransactions
          .concat(toTransactions)
          .map((transaction) => {
            return {
              blockNumber: transaction.blockNumber,
              from: transaction.action.from,
              hash: transaction.transactionHash,
              to: transaction.action.to
            };
          });
      });
  }
}
