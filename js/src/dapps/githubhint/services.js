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

import * as abis from '../../contracts/abi';
import { api } from './parity';

export function attachInterface () {
  return api.ethcore
    .registryAddress()
    .then((registryAddress) => {
      console.log(`the registry was found at ${registryAddress}`);

      const registry = api.newContract(abis.registry, registryAddress).instance;

      return Promise
        .all([
          registry.getAddress.call({}, [api.util.sha3('githubhint'), 'A']),
          api.eth.accounts(),
          null // api.personal.accountsInfo()
        ]);
    })
    .then(([address, addresses, accountsInfo]) => {
      accountsInfo = accountsInfo || {};
      console.log(`githubhint was found at ${address}`);

      const contract = api.newContract(abis.githubhint, address);
      const accounts = addresses.reduce((obj, address) => {
        const info = accountsInfo[address] || {};

        return Object.assign(obj, {
          [address]: {
            address,
            name: info.name,
            uuid: info.uuid
          }
        });
      }, {});
      const fromAddress = Object.keys(accounts)[0];

      return {
        accounts,
        address,
        accountsInfo,
        contract,
        instance: contract.instance,
        fromAddress
      };
    })
    .catch((error) => {
      console.error('attachInterface', error);
    });
}
