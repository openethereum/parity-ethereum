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

import { stringify } from 'qs';

const options = {
  method: 'GET',
  headers: {
    'Accept': 'application/json'
  }
};

export function call (module, action, _params, test, netVersion) {
  let prefix = 'api.';

  switch (netVersion) {
    case '2':
    case '3':
      prefix = 'testnet.';
      break;

    case '42':
      prefix = 'kovan.';
      break;

    case '0':
    default:
      if (test) {
        prefix = 'testnet.';
      }
      break;
  }

  const query = stringify(Object.assign({
    module, action
  }, _params || {}));

  return fetch(`https://${prefix}etherscan.io/api?${query}`, options)
    .then((response) => {
      if (!response.ok) {
        throw { code: response.status, message: response.statusText }; // eslint-disable-line
      }

      return response.json();
    })
    .then((result) => {
      if (result.message === 'NOTOK') {
        throw { code: -1, message: result.result }; // eslint-disable-line
      }

      return result.result;
    });
}
