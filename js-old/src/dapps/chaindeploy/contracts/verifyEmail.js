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

import abi from '~/contracts/abi/email-verification';
import { compiler, source as sourceUrl, output as byteCode } from './code/verifyEmail';

const isBadge = true;
const id = 'emailverification';
const deployParams = [];
const badgeSource = {
  imageUrl: 'https://raw.githubusercontent.com/paritytech/dapp-assets/c4721a87cb95375da91f8699438d8d7907b3f5e9/certifications/email-verification.svg',
  imageHash: '0x5617a14da2a0c210939da6eafb734e60906f64a504c3e107812668860a752dc6'
};

export {
  abi,
  badgeSource,
  byteCode,
  compiler,
  deployParams,
  id,
  isBadge,
  sourceUrl
};
