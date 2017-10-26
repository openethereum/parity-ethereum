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

const id = 'dappreg';
const app = builtins.find((app) => app.url === id);
const hashId = app.id;
const source = {
  imageUrl: 'https://raw.githubusercontent.com/paritytech/dapp-assets/cdd6ac4f1e2f11619bed72a53ae71217dffe19ad/dapps/legos-64x64.png',
  imageHash: '0xa8feea35c761cc6c2fe862fe336419f11ca5421f578757720a899b89fc1df154'
};
const name = app.name;

export {
  hashId,
  id,
  name,
  source
};
