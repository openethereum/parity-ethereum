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

const id = 'web';
const app = builtins.find((app) => app.url === id);
const hashId = app.id;
const source = {
  imageUrl: 'https://raw.githubusercontent.com/paritytech/dapp-assets/ec6138115d0e1f45258969cd90b3b274e0ff2258/dapps/earth-64x64.jpg',
  imageHash: '0x0b9b62a9262f75461191d4e8bf686c56528cbc0fe885c1f5878052ca8b2f65bf'
};
const name = app.name;

export {
  hashId,
  id,
  name,
  source
};
