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

import Contracts from '~/contracts';
import subscribeToEvents from '~/util/subscribe-to-events';

import registryABI from '~/contracts/abi/registry.json';

import { setReverse, startCachingReverses } from './actions';

export default (api) => (store) => {
  let contract, subscription, timeout, interval;

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
          .then((reverse) => store.dispatch(setReverse(address, reverse)));
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

        registry.getInstance()
          .then((instance) => api.newContract(registryABI, instance.address))
          .then((_contract) => {
            contract = _contract;

            subscription = subscribeToEvents(_contract, ['ReverseConfirmed', 'ReverseRemoved']);
            subscription.on('log', onLog);

            timeout = setTimeout(checkReverses, 5000);
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

        break;
      default:
        next(action);
    }
  };
};
