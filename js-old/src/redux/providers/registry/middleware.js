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

import { debounce } from 'lodash';
import lsstore from 'store';
import Contracts from '~/contracts';
import subscribeToEvents from '~/util/subscribe-to-events';

import registryABI from '~/contracts/abi/registry.json';

import { setReverse, startCachingReverses } from './actions';

const STORE_KEY = '_parity::reverses';

export default class RegistryMiddleware {
  contract;
  interval;
  store;
  subscription;
  timeout;

  addressesToCheck = {};

  constructor (api) {
    this._api = api;
  }

  toMiddleware () {
    return (store) => {
      this.store = store;

      return (next) => (action) => {
        switch (action.type) {
          case 'initAll':
            next(action);
            store.dispatch(startCachingReverses());
            break;

          case 'startCachingReverses':
            this.cacheReverses();
            break;

          case 'stopCachingReverses':
            if (this.subscription) {
              this.subscription.unsubscribe();
            }
            if (this.interval) {
              clearInterval(this.interval);
            }
            if (this.timeout) {
              clearTimeout(this.timeout);
            }

            this.write.flush();
            break;

          case 'setReverse':
            this.write(
              () => store.getState().nodeStatus.netChain,
              () => store.getState().registry.reverse,
              () => +store.getState().nodeStatus.blockNumber
            );
            next(action);
            break;

          default:
            next(action);
        }
      };
    };
  }

  cacheReverses () {
    const { registry } = Contracts.get();
    const cached = this.read(this.store.getState().nodeStatus.netChain);

    if (cached) {
      Object
        .entries(cached.reverses)
        .forEach(([ address, reverse ]) => this.store.dispatch(setReverse(address, reverse)));
    }

    registry.getInstance()
      .then((instance) => this._api.newContract(registryABI, instance.address))
      .then((_contract) => {
        this.contract = _contract;

        this.subscription = subscribeToEvents(this.contract, [
          'ReverseConfirmed', 'ReverseRemoved'
        ], {
          from: cached ? cached.lastBlock : 0
        });
        this.subscription.on('log', this.onLog);

        this.timeout = setTimeout(this.checkReverses, 10000);
        this.interval = setInterval(this.checkReverses, 20000);
      })
      .catch((err) => {
        console.error('Failed to start caching reverses:', err);
        throw err;
      });
  }

  checkReverses = () => {
    Object
      .keys(this.addressesToCheck)
      .forEach((address) => {
        this.contract
          .instance
          .reverse
          .call({}, [ address ])
          .then((reverse) => {
            this.store.dispatch(setReverse(address, reverse));
          });
      });

    this.addressesToCheck = {};
  };

  onLog = (log) => {
    switch (log.event) {
      case 'ReverseConfirmed':
        this.addressesToCheck[log.params.reverse.value] = true;

        break;
      case 'ReverseRemoved':
        delete this.addressesToCheck[log.params.reverse.value];

        break;
    }
  };

  read = (chain) => {
    const reverses = lsstore.get(`${STORE_KEY}::${chain}::data`);
    const lastBlock = lsstore.get(`${STORE_KEY}::${chain}::lastBlock`);

    if (!reverses || !lastBlock) {
      return null;
    }
    return { reverses, lastBlock };
  };

  write = debounce((getChain, getReverses, getLastBlock) => {
    const chain = getChain();
    const reverses = getReverses();
    const lastBlock = getLastBlock();

    lsstore.set(`${STORE_KEY}::${chain}::data`, reverses);
    lsstore.set(`${STORE_KEY}::${chain}::lastBlock`, lastBlock);
  }, 20000);
}
