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

import { getBalances, getTokens } from './balancesActions';
import { setAddressImage } from './imagesActions';

import * as abis from '../../contracts/abi';

import imagesEthereum from '../../../assets/images/contracts/ethereum-black-64x64.png';

const ETH = {
  name: 'Ethereum',
  tag: 'ETH',
  image: imagesEthereum
};

export default class Balances {
  constructor (store, api) {
    this._api = api;
    this._store = store;
    this._accountsInfo = null;
  }

  start () {
    this._subscribeBlockNumber();
    this._subscribeAccountsInfo();
  }

  _subscribeAccountsInfo () {
    this._api
      .subscribe('personal_accountsInfo', (error, accountsInfo) => {
        if (error) {
          return;
        }

        this._accountsInfo = accountsInfo;
        this._retrieveBalances();
      })
      .then((subscriptionId) => {
        console.log('balances._subscribeAccountsInfo', 'subscriptionId', subscriptionId);
      });
  }

  _subscribeBlockNumber () {
    this._api
      .subscribe('eth_blockNumber', (error) => {
        if (error) {
          return;
        }

        this._retrieveTokens();
      })
      .then((subscriptionId) => {
        console.log('balances._subscribeBlockNumber', 'subscriptionId', subscriptionId);
      });
  }

  _retrieveTokens () {
    this._api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        const registry = this._api.newContract(abis.registry, registryAddress);

        return registry.instance.getAddress.call({}, [this._api.util.sha3('tokenreg'), 'A']);
      })
      .then((tokenregAddress) => {
        const tokenreg = this._api.newContract(abis.tokenreg, tokenregAddress);

        return tokenreg.instance.tokenCount
          .call()
          .then((numTokens) => {
            const promisesTokens = [];
            const promisesImages = [];

            while (promisesTokens.length < numTokens.toNumber()) {
              const index = promisesTokens.length;

              promisesTokens.push(tokenreg.instance.token.call({}, [index]));
              promisesImages.push(tokenreg.instance.meta.call({}, [index, 'IMG']));
            }

            return Promise.all([
              Promise.all(promisesTokens),
              Promise.all(promisesImages)
            ]);
          });
      })
      .then(([_tokens, images]) => {
        const tokens = {};
        this._tokens = _tokens.map((_token, index) => {
          const [address, tag, format, name] = _token;

          const token = {
            address,
            name,
            tag,
            format: format.toString(),
            contract: this._api.newContract(abis.eip20, address)
          };
          tokens[address] = token;
          this._store.dispatch(setAddressImage(address, images[index]));

          return token;
        });

        this._store.dispatch(getTokens(tokens));
        this._retrieveBalances();
      })
      .catch((error) => {
        console.error('balances._retrieveTokens', error);
      });
  }

  _retrieveBalances () {
    if (!this._accountsInfo || !this._tokens) {
      return;
    }

    const addresses = Object.keys(this._accountsInfo);
    this._balances = {};

    Promise
      .all(
        addresses.map((address) => Promise.all([
          this._api.eth.getBalance(address),
          this._api.eth.getTransactionCount(address)
        ]))
      )
      .then((balanceTxCount) => {
        return Promise.all(
          balanceTxCount.map(([value, txCount], idx) => {
            const address = addresses[idx];

            this._balances[address] = {
              txCount,
              tokens: [{
                token: ETH,
                value
              }]
            };

            return Promise.all(
              this._tokens.map((token) => {
                return token.contract.instance.balanceOf.call({}, [address]);
              })
            );
          })
        );
      })
      .then((tokenBalances) => {
        addresses.forEach((address, idx) => {
          const balanceOf = tokenBalances[idx];
          const balance = this._balances[address];

          this._tokens.forEach((token, tidx) => {
            balance.tokens.push({
              token,
              value: balanceOf[tidx]
            });
          });
        });

        this._store.dispatch(getBalances(this._balances));
      })
      .catch((error) => {
        console.error('balances._retrieveBalances', error);
      });
  }
}
