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
import store from 'store';
import Contracts from '~/contracts';
import subscribeToEvents from '~/util/subscribe-to-events';

import registryABI from '~/contracts/abi/registry.json';

import { setReverse, startCachingReverses } from './actions';

const STORE_KEY = '_parity::reverses';

const read = (chain) => {
  const reverses = store.get(`${STORE_KEY}::${chain}::data`);
  const lastBlock = store.get(`${STORE_KEY}::${chain}::lastBlock`);

  if (!reverses || !lastBlock) {
    return null;
  }
  return { reverses, lastBlock };
};

const write = debounce((getChain, getReverses, getLastBlock) => {
  const chain = getChain();
  const reverses = getReverses();
  const lastBlock = getLastBlock();

  store.set(`${STORE_KEY}::${chain}::data`, reverses);
  store.set(`${STORE_KEY}::${chain}::lastBlock`, lastBlock);
}, 20000);

export default (api) => (store) => {
  let contract;
  let subscription;
  let timeout;
  let interval;

  let addressesToCheck = {};

  const onLog = (log) => {
    switch (log.event) {
      case 'ReverseConfirmed':
        addressesToCheck[log.params.reverse.value] = true;

        break;
      case 'ReverseRemoved':
        delete addressesToCheck[log.params.reverse.value];

        break;
    }
  };

  const checkReverses = () => {
    Object
      .keys(addressesToCheck)
      .forEach((address) => {
        contract
          .instance
          .reverse
          .call({}, [ address ])
          .then((reverse) => {
            store.dispatch(setReverse(address, reverse));
          });
      });

    addressesToCheck = {};
  };

  return (next) => (action) => {
    switch (action.type) {
      case 'initAll':
        next(action);
        store.dispatch(startCachingReverses());

        break;

      case 'startCachingReverses':
        const { registry } = Contracts.get();
        const cached = read(store.getState().nodeStatus.netChain);

        if (cached) {
          Object
            .entries(cached.reverses)
            .forEach(([ address, reverse ]) => store.dispatch(setReverse(address, reverse)));
        }

        registry.getInstance()
          .then((instance) => api.newContract(registryABI, instance.address))
          .then((_contract) => {
            contract = _contract;

            subscription = subscribeToEvents(_contract, [
              'ReverseConfirmed', 'ReverseRemoved'
            ], {
              from: cached ? cached.lastBlock : 0
            });
            subscription.on('log', onLog);

            timeout = setTimeout(checkReverses, 10000);
            interval = setInterval(checkReverses, 20000);
          })
          .catch((err) => {
            console.error('Failed to start caching reverses:', err);
            throw err;
          });

        break;
      case 'stopCachingReverses':
        if (subscription) {
          subscription.unsubscribe();
        }
        if (interval) {
          clearInterval(interval);
        }
        if (timeout) {
          clearTimeout(timeout);
        }

        write.flush();
        break;

      case 'setReverse':
        write(
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
