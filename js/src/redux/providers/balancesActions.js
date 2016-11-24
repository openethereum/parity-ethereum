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

import { range, uniq } from 'lodash';

import { hashToImageUrl } from './imagesReducer';
import { setAddressImage } from './imagesActions';

import * as ABIS from '../../contracts/abi';
import imagesEthereum from '../../../assets/images/contracts/ethereum-black-64x64.png';

const ETH = {
  name: 'Ethereum',
  tag: 'ETH',
  image: imagesEthereum
};

export function setBalances (balances) {
  return {
    type: 'setBalances',
    balances
  };
}

export function setTokens (tokens) {
  return {
    type: 'setTokens',
    tokens
  };
}

export function setTokenReg (tokenreg) {
  return {
    type: 'setTokenReg',
    tokenreg
  };
}

export function loadTokens () {
  return (dispatch, getState) => {
    const { tokenreg } = getState().balances;

    return tokenreg.instance.tokenCount
      .call()
      .then((numTokens) => {
        const tokenIds = range(numTokens.toNumber());
        dispatch(fetchTokens(tokenIds));
      })
      .catch((error) => {
        console.warn('balances::loadTokens', error);
      });
  };
}

export function fetchTokens (_tokenIds) {
  const tokenIds = uniq(_tokenIds || []);

  return (dispatch, getState) => {
    const { api, images, balances } = getState();
    const { tokenreg } = balances;

    return Promise
      .all(tokenIds.map((id) => fetchTokenInfo(tokenreg, id, api)))
      .then((tokens) => {
        // dispatch only the changed images
        tokens
          .forEach((token) => {
            const { image, address } = token;

            if (images[address] === image) {
              return;
            }

            dispatch(setAddressImage(address, image, true));
          });

        dispatch(setTokens(tokens));
        dispatch(fetchBalances());
      })
      .catch((error) => {
        console.warn('balances::fetchTokens', error);
      });
  };
}

export function fetchBalances (_addresses) {
  return (dispatch, getState) => {
    const { api, balances, personal } = getState();
    const { visibleAccounts } = personal;
    const tokens = Object.values(balances.tokens) || [];

    const addresses = uniq(_addresses || visibleAccounts || []);

    return Promise
      .all(addresses.map((addr) => fetchAccount(addr, tokens, api)))
      .then((_balances) => {
        const balances = {};

        addresses.forEach((addr, idx) => {
          balances[addr] = _balances[idx];
        });

        dispatch(setBalances(balances));
      })
      .catch((error) => {
        console.warn('balances::fetchBalances', error);
      });
  };
}

function fetchAccount (address, _tokens, api) {
  const tokensPromises = _tokens
    .map((token) => {
      return token.contract.instance.balanceOf.call({}, [ address ]);
    });

  return Promise
    .all([
      api.eth.getTransactionCount(address),
      api.eth.getBalance(address)
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
    })
    .catch((error) => {
      console.warn('balances::fetchAccountBalance', `couldn't fetch balance for account #${address}`, error);
    });
}

function fetchTokenInfo (tokenreg, tokenId, api, dispatch) {
  return Promise
    .all([
      tokenreg.instance.token.call({}, [tokenId]),
      tokenreg.instance.meta.call({}, [tokenId, 'IMG'])
    ])
    .then(([ tokenData, image ]) => {
      const [ address, tag, format, name ] = tokenData;
      const contract = api.newContract(ABIS.eip20, address);

      const token = {
        format: format.toString(),
        id: tokenId,
        image: hashToImageUrl(image),

        address,
        tag,
        name,
        contract
      };

      return token;
    })
    .catch((error) => {
      console.warn('balances::fetchTokenInfo', `couldn't fetch token #${tokenId}`, error);
    });
}
