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

import parity from '@parity/jsonrpc/interfaces/parity';
import signer from '@parity/jsonrpc/interfaces/signer';
import trace from '@parity/jsonrpc/interfaces/trace';

export default function web3extensions (web3) {
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
