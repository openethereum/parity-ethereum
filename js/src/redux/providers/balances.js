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
    this._tokens = {};
    this._images = {};
    this._tokenreg = null;
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
        console.log('_subscribeAccountsInfo', 'subscriptionId', subscriptionId);
      })
      .catch((error) => {
        console.warn('_subscribeAccountsInfo', error);
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
        console.log('_subscribeBlockNumber', 'subscriptionId', subscriptionId);
      })
      .catch((error) => {
        console.warn('_subscribeBlockNumber', error);
      });
  }

  getTokenRegistry () {
    if (this._tokenreg) {
      return Promise.resolve(this._tokenreg);
    }

    return this._api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        const registry = this._api.newContract(abis.registry, registryAddress);

        return registry.instance.getAddress.call({}, [this._api.util.sha3('tokenreg'), 'A']);
      })
      .then((tokenregAddress) => {
        const tokenreg = this._api.newContract(abis.tokenreg, tokenregAddress);
        this._tokenreg = tokenreg;

        return tokenreg;
      });
  }

  _retrieveTokens () {
    this
      .getTokenRegistry()
      .then((tokenreg) => {
        return tokenreg.instance.tokenCount
          .call()
          .then((numTokens) => {
            const promises = [];

            for (let i = 0; i < numTokens.toNumber(); i++) {
              promises.push(this.fetchTokenInfo(tokenreg, i));
            }

            return Promise.all(promises);
          });
      })
      .then((_tokens) => {
        const prevHashes = Object.values(this._tokens).map((t) => t.hash).sort().join('');
        const nextHashes = _tokens.map((t) => t.hash).sort().join('');

        if (prevHashes !== nextHashes) {
          this._tokens = _tokens
            .reduce((obj, token) => {
              obj[token.address] = token;
              return obj;
            }, {});

          this._store.dispatch(getTokens(this._tokens));
        }

        this._retrieveBalances();
      })
      .catch((error) => {
        console.warn('_retrieveTokens', error);
        this._retrieveBalances();
      });
  }

  _retrieveBalances () {
    if (!this._accountsInfo) {
      return;
    }

    const addresses = Object.keys(this._accountsInfo);
    this._balances = {};

    Promise
      .all(addresses.map((a) => this.fetchAccountBalance(a)))
      .then((balances) => {
        addresses.forEach((a, idx) => {
          this._balances[a] = balances[idx];
        });

        this._store.dispatch(getBalances(this._balances));
      })
      .catch((error) => {
        console.warn('_retrieveBalances', error);
      });
  }

  fetchTokenInfo (tokenreg, tokenId) {
    return Promise
      .all([
        tokenreg.instance.token.call({}, [tokenId]),
        tokenreg.instance.meta.call({}, [tokenId, 'IMG'])
      ])
      .then(([ token, image ]) => {
        const [ address, tag, format, name ] = token;
        const oldToken = this._tokens[address];

        if (this._images[address] !== image.toString()) {
          this._store.dispatch(setAddressImage(address, image));
          this._images[address] = image.toString();
        }

        const newToken = {
          address,
          name,
          tag,
          format: format.toString()
        };

        const hash = this._api.util.sha3(JSON.stringify(newToken));

        const contract = oldToken
          ? oldToken.contract
          : this._api.newContract(abis.eip20, address);

        return {
          ...newToken,
          hash,
          contract
        };
      });
  }

  fetchAccountBalance (address) {
    const _tokens = Object.values(this._tokens);
    const tokensPromises = _tokens
      .map((token) => {
        return token.contract.instance.balanceOf.call({}, [ address ]);
      });

    return Promise
      .all([
        this._api.eth.getTransactionCount(address),
        this._api.eth.getBalance(address)
      ].concat(tokensPromises))
      .then(([ txCount, ethBalance, ...tokensBalance ]) => {
        const tokens = _tokens
          .map((token, index) => ({
            token,
            value: tokensBalance[index]
          }))
          .concat({
            token: ETH,
            value: ethBalance
          });

        const balance = { txCount, tokens };
        return balance;
      });
  }
}
