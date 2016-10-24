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

import { personalAccountsInfo } from './personalActions';
import { statusNewTransactions } from './statusActions';
import { isEqual } from 'lodash';

export default class Personal {
  constructor (store, api) {
    this._api = api;
    this._store = store;

    this._addresses = [];
  }

  start () {
    this._subscribeAccountsInfo();

    this._api
      .subscribe('eth_blockNumber', () => {
        const addresses = this._addresses;

        if (!addresses || addresses.length === 0) {
          return;
        }

        Promise
          .all([
            this._api.trace.filter({ fromAddress: addresses }),
            this._api.trace.filter({ toAddress: addresses })
          ])
          .then(([ fromTraces, toTraces ]) => {
            const traces = Object.values([]
              .concat(fromTraces, toTraces)
              .reduce((txs, trace) => {
                txs[trace.transactionHash] = trace;
              }, {}));

            if (traces.length === 0) {
              return;
            }

            const transactions = traces.map(transaction => ({
              from: transaction.action.from,
              to: transaction.action.to,
              blockNumber: transaction.blockNumber,
              hash: transaction.transactionHash
            }));

            this._store.dispatch(statusNewTransactions(transactions));
          })
          .catch(e => {
            console.error('personal::trace_filter', e);
          });
      });
  }

  _subscribeAccountsInfo () {
    this._api
      .subscribe('personal_accountsInfo', (error, accountsInfo) => {
        if (error) {
          console.error('personal_accountsInfo', error);
          return;
        }

        if (accountsInfo) {
          const addresses = Object.keys(accountsInfo).sort();

          if (!isEqual(addresses, this._addresses)) {
            this._addresses = addresses;
          }
        }

        this._store.dispatch(personalAccountsInfo(accountsInfo));
      })
      .then((subscriptionId) => {
        console.log('personal._subscribeAccountsInfo', 'subscriptionId', subscriptionId);
      });
  }
}
