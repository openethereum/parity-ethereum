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
import es6Promise from 'es6-promise';
import parity from 'parity/jsonrpc/interfaces/parity';
import signer from 'parity/jsonrpc/interfaces/signer';
import trace from 'parity/jsonrpc/interfaces/trace';
/** PARITY **/
import Api from 'parity/api';
import './dev.parity.html';
/** WEB3 **/
import Web3 from 'web3';
import './dev.web3.html';
/** WEB3PROVIDER **/
const web3Provider = new Api.Provider.Http('/rpc/');

es6Promise.polyfill();

/** INJECT PARITY **/
const api = new Api(web3Provider);

window.parity = {
  Api,
  api,
  web3Provider
};

/** INJECT WEB3 **/
const http = new Web3.providers.HttpProvider('/rpc/');
const web3 = new Web3(http);

// set default account
web3.eth.getAccounts((err, accounts) => {
  if (err || !accounts || !accounts[0]) {
    return;
  }

  web3.eth.defaultAccount = accounts[0];
});

web3extensions(web3).map((extension) => web3._extend(extension));

global.web3 = web3;

function web3extensions (web3) {
  const { Method } = web3._extend;

  // TODO [ToDr] Consider output/input formatters.
  const methods = (object, name) => {
    return Object.keys(object).map(method => {
      return new Method({
        name: method,
        call: `${name}_{method}`,
        params: object[method].params.length
      });
    });
  };

  return [{
    property: 'parity',
    methods: methods(parity, 'parity'),
    properties: []
  }, {
    property: 'signer',
    methods: methods(signer, 'signer'),
    properties: []
  }, {
    property: 'trace',
    methods: methods(trace, 'trace'),
    properties: []
  }];
}
