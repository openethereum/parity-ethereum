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
const id = 'jg-voting';
const hashId = api.util.sha3(id);
const source = {
  imageUrl: 'https://raw.githubusercontent.com/jacogr/dapp-voting/038ff4074544f2babc7aed9c4ac3dc070b85b804/assets/images/vote.jpg',
  imageHash: '0x3620828e1a745d2714e9f37dc2d47cdab5ef9982190a845b5e7665b7a7767661',
  manifestUrl: 'https://raw.githubusercontent.com/jacogr/dapp-voting/682f0fe4b86508a1a2487de6c7c61f7b100ba5b2/manifest.json',
  manifestHash: '0x9b83e01f87d225e7bfdd305c51319504ff9b4cf8d517ca4b64f606762e72f59e',
  contentUrl: {
    repo: 'jacogr/dapp-voting',
    commit: '0x7d941597e862a600a60b9d6ecd3a6f606d96cd7b'
  },
  contentHash: '0x9fcc0910f6a8c4e45715d41aea2d287da31bf1d7321003fc66df6a012ce2d753'
};
const name = 'Yes, No, Maybe';

export {
  hashId,
  id,
  isExternal,
  name,
  source
};
