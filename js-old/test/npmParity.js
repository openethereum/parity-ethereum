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

try {
  var Api = require('../.npmjs/parity/library.js').Api;
  var Abi = require('../.npmjs/parity/library.js').Abi;

  if (typeof Api !== 'function') {
    throw new Error('No Api');
  }

  if (typeof Abi !== 'function') {
    throw new Error('No Abi');
  }

  var transport = new Api.Transport.Http('http://localhost:8545');
  var api = new Api(transport);

  api.eth
    .blockNumber()
    .then((block) => {
      console.log('library working fine', '(block #' + block.toFormat() + ')');
      process.exit(0);
    })
    .catch(() => {
      console.log('library working fine (disconnected)');
      process.exit(0);
    });
} catch (e) {
  console.error('An error occured:', e.toString().split('\n')[0]);
  process.exit(1);
}
