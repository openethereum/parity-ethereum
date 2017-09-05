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

import { url as etherscanUrl } from '~/3rdparty/etherscan/links';
import * as abis from '~/contracts/abi';
import { api } from './parity';

let managerInstance;
let tokenregInstance;
let registryInstance;

const registries = {};
const subscriptions = {};

let defaultSubscriptionId;
let nextSubscriptionId = 1000;
let netVersion = '0';

export function subscribeEvents (addresses, callback) {
  const subscriptionId = nextSubscriptionId++;
  const contract = api.newContract(abis.eip20);
  const event = contract.events.filter((evt) => evt.name === 'Transfer');
  const options = {
    address: addresses,
    fromBlock: 0,
    toBlock: 'pending',
    limit: 50,
    topics: [event.signature]
  };

  return api.eth
    .newFilter(options)
    .then((filterId) => {
      subscriptions[subscriptionId] = { subscriptionId, filterId, addresses, callback, contract };

      return api.eth.getFilterLogs(filterId);
    })
    .then((logs) => callback(null, contract.parseEventLogs(logs)))
    .then(() => subscriptionId)
    .catch((error) => {
      console.error('subscribeEvents', error);
      throw error;
    });
}

export function unsubscribeEvents (subscriptionId) {
  api.eth
    .uninstallFilter(subscriptions[subscriptionId].filterId)
    .catch((error) => {
      console.error('unsubscribeEvents', error);
    });

  delete subscriptions[subscriptionId];
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

function pollEvents () {
  const loop = Object.values(subscriptions);
  const timeout = () => setTimeout(pollEvents, 1000);

  Promise
    .all(loop.map((subscription) => api.eth.getFilterChanges(subscription.filterId)))
    .then((logsArray) => {
      logsArray.forEach((logs, index) => {
        const subscription = loop[index];

        if (!logs || !logs.length) {
          return;
        }

        try {
          subscription.callback(null, subscription.contract.parseEventLogs(logs));
        } catch (error) {
          unsubscribeEvents(loop.subscriptionId);
          console.error('pollEvents', error);
        }
      });

      timeout();
    })
    .catch((error) => {
      console.error('pollEvents', error);
      timeout();
    });
}

export function attachInstances () {
  pollEvents();

  return Promise
    .all([
      api.parity.registryAddress(),
      api.parity.netChain(),
      api.net.version()
    ])
    .then(([registryAddress, netChain, _netVersion]) => {
      const registry = api.newContract(abis.registry, registryAddress).instance;

      netVersion = _netVersion;

      console.log(`contract was found at registry=${registryAddress}`);
      console.log(`running on ${netChain}, network ${netVersion}`);

      return Promise
        .all([
          registry.getAddress.call({}, [api.util.sha3('basiccoinmgr'), 'A']),
          registry.getAddress.call({}, [api.util.sha3('basiccoinreg'), 'A']),
          registry.getAddress.call({}, [api.util.sha3('tokenreg'), 'A'])
        ]);
    })
    .then(([managerAddress, registryAddress, tokenregAddress]) => {
      console.log(`contracts were found at basiccoinmgr=${managerAddress}, basiccoinreg=${registryAddress}, tokenreg=${tokenregAddress}`);

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

export function loadOwnedTokens (addresses) {
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

export function loadAllTokens () {
  return managerInstance
    .count.call()
    .then((count) => {
      const promises = [];

      for (let index = 0; count.gt(index); index++) {
        promises.push(managerInstance.get.call({}, [index]));
      }

      return Promise.all(promises);
    })
    .then((_tokens) => {
      const tokens = [];

      return Promise
        .all(
          _tokens.map(([address, owner, tokenreg]) => {
            const isGlobal = tokenreg === tokenregInstance.address;

            tokens.push({ address, owner, tokenreg, isGlobal });
            return registries[tokenreg].fromAddress.call({}, [address]);
          })
        )
        .then((coins) => {
          return tokens.map((token, index) => {
            const [id, tla, base, name, owner] = coins[index];

            token.coin = { id, tla, base, name, owner };
            return token;
          });
        });
    })
    .catch((error) => {
      console.log('loadAllTokens', error);
      throw error;
    });
}

export function loadBalances (addresses) {
  return loadAllTokens()
    .then((tokens) => {
      return Promise.all(
        tokens.map((token) => {
          return Promise.all(
            addresses.map((address) => loadTokenBalance(token.address, address))
          );
        })
      )
      .then((_balances) => {
        return tokens.map((token, tindex) => {
          const balances = _balances[tindex];

          token.balances = addresses.map((address, aindex) => {
            return { address, balance: balances[aindex] };
          });
          return token;
        });
      });
    })
    .catch((error) => {
      console.error('loadBalances', error);
      throw error;
    });
}

export function loadTokenBalance (tokenAddress, address) {
  return api.newContract(abis.eip20, tokenAddress).instance
    .balanceOf.call({}, [address])
    .catch((error) => {
      console.error('loadTokenBalance', error);
      throw error;
    });
}

export function txLink (txHash) {
  return `https://${etherscanUrl(false, netVersion)}/tx/${txHash}`;
}
