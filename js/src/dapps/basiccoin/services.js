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

import BigNumber from 'bignumber.js';

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
    })
    .catch((error) => {
      console.error('getCoin', error);
      throw error;
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
    })
    .catch((error) => {
      console.error('attachInstances', error);
      throw error;
    });
}

export function loadTokens (addresses) {
  let total = new BigNumber(0);

  return Promise
    .all(
      addresses.map((address) => managerInstance.countByOwner.call({}, [address]))
    )
    .then((counts) => {
      return Promise.all(
        addresses.reduce((promises, address, index) => {
          total = counts[index].add(total);
          for (let i = 0; counts[index].gt(i); i++) {
            promises.push(managerInstance.getByOwner.call({}, [address, i]));
          }
          return promises;
        }, [])
      );
    })
    .then((_tokens) => {
      const tokens = _tokens.reduce((tokens, token) => {
        const [address, owner, tokenreg] = token;
        tokens[owner] = tokens[owner] || [];
        tokens[owner].push({ address, owner, tokenreg });
        return tokens;
      }, {});

      return { tokens, total };
    })
    .catch((error) => {
      console.error('loadTokens', error);
      throw error;
    });
}

export function loadBalances (addresses) {
  return Promise
    .all([
      loadInstanceBalances(tokenregInstance, addresses),
      loadInstanceBalances(registryInstance, addresses)
    ])
    .then(([trBalances, bcBalances]) => {
      return {
        global: trBalances,
        local: bcBalances
      };
    })
    .catch((error) => {
      console.error('loadBalances', error);
      throw error;
    });
}

export function loadInstanceBalances (tokenreg, addresses) {
  return loadInstanceCoins(tokenreg)
    .then((coins) => {
      return Promise.all(
        coins.map((coin) => {
          return Promise.all(
             addresses.map((address) => loadCoinBalance(coin.address, address))
          );
        })
      )
      .then((_balances) => {
        return _balances.map((_balance, cindex) => {
          return {
            coin: coins[cindex],
            balances: _balance.reduce((balance, value, aindex) => {
              balance[addresses[aindex]] = value;
              return balance;
            }, {})
          };
        });
      });
    })
    .catch((error) => {
      console.error('loadInstanceBalances', error);
      throw error;
    });
}

export function loadCoinBalance (coinAddress, address) {
  return api.newContract(abis.eip20, coinAddress).instance
    .balanceOf.call({}, [address])
    .catch((error) => {
      console.error('loadCoinBalance', error);
      throw error;
    });
}

export function loadInstanceCoins (tokenreg) {
  return tokenreg
    .tokenCount.call()
    .then((count) => {
      const promises = [];
      for (let i = 0; count.gt(i); i++) {
        promises.push(tokenreg.token.call({}, [i]));
      }
      return Promise.all(promises);
    })
    .then((coins) => {
      return coins.map(([address, tla, base, name, owner], id) => {
        return { id, address, tla, base, name, owner };
      });
    })
    .catch((error) => {
      console.error('loadInstanceCoins', error);
      throw error;
    });
}
