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

import Contracts from '../../contracts';
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

    this._tokens = {};
    this._images = {};

    this._accountsInfo = null;
    this._tokenreg = null;
    this._fetchingTokens = false;
    this._fetchedTokens = false;

    this._tokenregSubId = null;
    this._tokenregMetaSubId = null;
  }

  start () {
    this._subscribeBlockNumber();
    this._subscribeAccountsInfo();
    this._retrieveTokens();
  }

  _subscribeAccountsInfo () {
    this._api
      .subscribe('parity_accountsInfo', (error, accountsInfo) => {
        if (error) {
          return;
        }

        this._accountsInfo = accountsInfo;
        this._retrieveTokens();
      })
      .catch((error) => {
        console.warn('_subscribeAccountsInfo', error);
      });
  }

  _subscribeBlockNumber () {
    this._api
      .subscribe('eth_blockNumber', (error) => {
        if (error) {
          return console.warn('_subscribeBlockNumber', error);
        }

        this._retrieveTokens();
      })
      .catch((error) => {
        console.warn('_subscribeBlockNumber', error);
      });
  }

  getTokenRegistry () {
    if (this._tokenreg) {
      return Promise.resolve(this._tokenreg);
    }

    return Contracts.get().tokenReg
      .getContract()
      .then((tokenreg) => {
        this._tokenreg = tokenreg;
        this.attachToTokens();

        return tokenreg;
      });
  }

  _retrieveTokens () {
    if (this._fetchingTokens) {
      return;
    }

    if (this._fetchedTokens) {
      return this._retrieveBalances();
    }

    this._fetchingTokens = true;
    this._fetchedTokens = false;

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
      .then(() => {
        this._fetchingTokens = false;
        this._fetchedTokens = true;

        this._store.dispatch(getTokens(this._tokens));
        this._retrieveBalances();
      })
      .catch((error) => {
        console.warn('balances::_retrieveTokens', error);
      });
  }

  _retrieveBalances () {
    if (!this._accountsInfo) {
      return;
    }

    const addresses = Object
      .keys(this._accountsInfo)
      .filter((address) => {
        const account = this._accountsInfo[address];
        return !account.meta || !account.meta.deleted;
      });

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

  attachToTokens () {
    this.attachToTokenMetaChange();
    this.attachToNewToken();
  }

  attachToNewToken () {
    if (this._tokenregSubId) {
      return;
    }

    this._tokenreg
      .instance
      .Registered
      .subscribe({
        fromBlock: 0,
        toBlock: 'latest',
        skipInitFetch: true
      }, (error, logs) => {
        if (error) {
          return console.error('balances::attachToNewToken', 'failed to attach to tokenreg Registered', error.toString(), error.stack);
        }

        const promises = logs.map((log) => {
          const id = log.params.id.value.toNumber();
          return this.fetchTokenInfo(this._tokenreg, id);
        });

        return Promise.all(promises);
      })
      .then((tokenregSubId) => {
        this._tokenregSubId = tokenregSubId;
      })
      .catch((e) => {
        console.warn('balances::attachToNewToken', e);
      });
  }

  attachToTokenMetaChange () {
    if (this._tokenregMetaSubId) {
      return;
    }

    this._tokenreg
      .instance
      .MetaChanged
      .subscribe({
        fromBlock: 0,
        toBlock: 'latest',
        topics: [ null, this._api.util.asciiToHex('IMG') ],
        skipInitFetch: true
      }, (error, logs) => {
        if (error) {
          return console.error('balances::attachToTokenMetaChange', 'failed to attach to tokenreg MetaChanged', error.toString(), error.stack);
        }

        // In case multiple logs for same token
        // in one block. Take the last value.
        const tokens = logs
          .filter((log) => log.type === 'mined')
          .reduce((_tokens, log) => {
            const id = log.params.id.value.toNumber();
            const image = log.params.value.value;

            const token = Object.values(this._tokens).find((c) => c.id === id);
            const { address } = token;

            _tokens[address] = { address, id, image };
            return _tokens;
          }, {});

        Object
          .values(tokens)
          .forEach((token) => {
            const { address, image } = token;

            if (this._images[address] !== image.toString()) {
              this._store.dispatch(setAddressImage(address, image));
              this._images[address] = image.toString();
            }
          });
      })
      .then((tokenregMetaSubId) => {
        this._tokenregMetaSubId = tokenregMetaSubId;
      })
      .catch((e) => {
        console.warn('balances::attachToTokenMetaChange', e);
      });
  }

  fetchTokenInfo (tokenreg, tokenId) {
    return Promise
      .all([
        tokenreg.instance.token.call({}, [tokenId]),
        tokenreg.instance.meta.call({}, [tokenId, 'IMG'])
      ])
      .then(([ tokenData, image ]) => {
        const [ address, tag, format, name ] = tokenData;
        const contract = this._api.newContract(abis.eip20, address);

        if (this._images[address] !== image.toString()) {
          this._store.dispatch(setAddressImage(address, image));
          this._images[address] = image.toString();
        }

        const token = {
          format: format.toString(),
          id: tokenId,

          address,
          tag,
          name,
          contract
        };

        this._tokens[address] = token;

        return token;
      })
      .catch((e) => {
        console.warn('balances::fetchTokenInfo', `couldn't fetch token #${tokenId}`, e);
      });
  }

  /**
   * TODO?: txCount is only shown on an address page, so we
   * might not need to fetch it for each address for each block,
   * but only for one address when the user is on the account
   * view.
   */
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
        const tokens = []
          .concat(
            { token: ETH, value: ethBalance },
            _tokens
              .map((token, index) => ({
                token,
                value: tokensBalance[index]
              }))
          );

        const balance = { txCount, tokens };
        return balance;
      });
  }
}
