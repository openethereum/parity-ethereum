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

let managerInstance;
let tokenregInstance;
let registryInstance;

const registries = {};

export function totalSupply (address) {
  return api.newContract(abis.eip20, address)
    .instance.totalSupply.call();
}

export function getCoin (tokenreg, address) {
  return registries[tokenreg].fromAddress
    .call({}, [address])
    .then(([id, tla, base, name, owner]) => {
      return {
        id, tla, base, name, owner,
        isGlobal: tokenregInstance.address === tokenreg
      };
    });
}

export function attachInstances () {
  return api.ethcore
    .registryAddress()
    .then((registryAddress) => {
      console.log(`contract was found at registry=${registryAddress}`);

      const registry = api.newContract(abis.registry, registryAddress).instance;

      return Promise
        .all([
          registry.getAddress.call({}, [api.util.sha3('playbasiccoinmgr'), 'A']),
          registry.getAddress.call({}, [api.util.sha3('basiccoinreg'), 'A']),
          registry.getAddress.call({}, [api.util.sha3('tokenreg'), 'A'])
        ]);
    })
    .then(([managerAddress, registryAddress, tokenregAddress]) => {
      console.log(`contracts were found at basiccoinmgr=${managerAddress}, basiccoinreg=${registryAddress}, tokenreg=${registryAddress}`);

      managerInstance = api.newContract(abis.basiccoinmanager, managerAddress).instance;
      registryInstance = api.newContract(abis.tokenreg, registryAddress).instance;
      tokenregInstance = api.newContract(abis.tokenreg, tokenregAddress).instance;

      registries[registryInstance.address] = registryInstance;
      registries[tokenregInstance.address] = tokenregInstance;

      return {
        managerInstance,
        registryInstance,
        tokenregInstance
      };
    });
}
