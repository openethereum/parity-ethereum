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

import { api } from '../parity';

const isExternal = true;
const id = 'gavcoin';
const hashId = api.util.sha3(id);
const source = {
  imageUrl: 'https://raw.githubusercontent.com/paritytech/dapp-assets/9e135f76fe9ba61e2d8ccbd72ed144c26c450780/tokens/gavcoin-64x64.png',
  imageHash: '0xd40679a3a234d8421c678d64f4df3308859e8ad07ac95ce4a228aceb96955287',
  manifestUrl: 'https://raw.githubusercontent.com/gavofyork/gavcoin/eb2f8dc4d3ad4dd5f4562690525b7cfedc9681ba/manifest.json',
  manifestHash: '0xd582c572fbef8015837f2c1a8798f2c3149a1d9d655b4020edb1bbece725371d',
  contentUrl: {
    repo: 'gavofyork/gavcoin',
    commit: '0xeb2f8dc4d3ad4dd5f4562690525b7cfedc9681ba'
  },
  contentHash: '0x0b6c7b3fc8dad3edb39fd1465904ce9a11938ef18f08f8115f275047ba249530'
};
const name = 'GavCoin';

export {
  hashId,
  id,
  isExternal,
  name,
  source
};
