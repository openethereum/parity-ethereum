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

const isExternal = true;
const id = 'whenarewethere';
const hashId = '0xfef3bfded03695e38a9ff476a0999e1fa421e72d1fb3b55a87d6c2bdb6fc18ef';
const source = {
  imageUrl: 'https://raw.githubusercontent.com/jacogr/dapp-when-are-we-there/167aa4d904c5aa6246d0d6d6f41c4ed8a56889cd/assets/images/clock.jpg',
  imageHash: '0x2534b44f685b6399bf63f86679128216c43e9a58be1dfb58533923f0bcffeba7',
  manifestUrl: 'https://raw.githubusercontent.com/jacogr/dapp-when-are-we-there/bf72dc3033711a3ab41bec3c1249638f70bae768/manifest.json',
  manifestHash: '0xfe26f6a19ea9393d69bc5d8c73c5072ccf126f51c10c135b42d6bf162d774fd9',
  contentUrl: {
    repo: 'jacogr/dapp-when-are-we-there',
    commit: '0xbf72dc3033711a3ab41bec3c1249638f70bae768'
  },
  contentHash: '0x3505cbbef5c2243eedba07d340d4abccfaa3cfb799f51827e33c9721a5254d13'
};
const name = 'When are we there';

export {
  hashId,
  id,
  isExternal,
  name,
  source
};
