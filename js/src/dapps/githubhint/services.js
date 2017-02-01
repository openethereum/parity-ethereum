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

import * as abis from '~/contracts/abi';
import { api } from './parity';

let defaultSubscriptionId;

export function attachInterface () {
  return api.parity
    .registryAddress()
    .then((registryAddress) => {
      console.log(`the registry was found at ${registryAddress}`);

      return api
        .newContract(abis.registry, registryAddress).instance
        .getAddress.call({}, [api.util.sha3('githubhint'), 'A']);
    })
    .then((address) => {
      console.log(`githubhint was found at ${address}`);

      const contract = api.newContract(abis.githubhint, address);

      return {
        address,
        contract,
        instance: contract.instance
      };
    })
    .catch((error) => {
      console.error('attachInterface', error);
    });
}

export function subscribeDefaultAddress (callback) {
  return api
    .subscribe('parity_defaultAccount', callback)
    .then((subscriptionId) => {
      defaultSubscriptionId = subscriptionId;

      return defaultSubscriptionId;
    });
}

export function unsubscribeDefaultAddress () {
  return api.unsubscribe(defaultSubscriptionId);
}
