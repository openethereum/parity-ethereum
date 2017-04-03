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

import builtins from '~/views/Dapps/builtin.json';

const id = 'console';
const app = builtins.find((app) => app.url === id);
const hashId = app.id;
const source = {
  imageUrl: 'https://raw.githubusercontent.com/paritytech/console/3ea0dbfefded359ccdbea37bc4cf350c0aa16948/console.jpeg',
  imageHash: '0xc3962e2eab7afaeb9cd11522381723afbafdc41dc7ba31bee472e187c4459e81'
};
const name = app.name;

export {
  hashId,
  id,
  name,
  source
};
