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

import 'whatwg-fetch';

import Api from '@parity/api';
import Web3 from 'web3';

import web3extensions from './web3.extensions';

function initProvider () {
  const parts = window.location.pathname.split('/');
  let appId = parts[1];

  if (appId === 'dapps') {
    appId = parts[2];
  } else if (!Api.util.isHex(appId)) {
    appId = Api.util.sha3(appId);
  }

  const ethereum = new Api.Provider.PostMessage(appId);

  console.log(`Requesting API communications token for ${appId}`);

  ethereum
    .requestNewToken()
    .then((tokenId) => {
      console.log(`Received API communications token ${tokenId}`);
    })
    .catch((error) => {
      console.error('Unable to retrieve communications token', error);
    });

  window.ethereum = ethereum;
  window.isParity = true;

  return ethereum;
}

function initWeb3 (ethereum) {
  // FIXME: Use standard provider for web3
  const http = new Web3.providers.HttpProvider('/rpc/');
  const web3 = new Web3(http);

  // set default account
  web3.eth.getAccounts((error, accounts) => {
    if (error || !accounts || !accounts[0]) {
      return;
    }

    web3.eth.defaultAccount = accounts[0];
  });

  web3extensions(web3).map((extension) => web3._extend(extension));

  window.web3 = web3;
}

function initParity (ethereum) {
  const api = new Api(ethereum);

  window.parity = Object.assign({}, window.parity || {}, {
    Api,
    api
  });
}

const ethereum = initProvider();

initWeb3(ethereum);
initParity(ethereum);

console.warn('Deprecation: Dapps should only used the exposed EthereumProvider on `window.ethereum`, the use of `window.parity` and `window.web3` will be removed in future versions of this injector');
