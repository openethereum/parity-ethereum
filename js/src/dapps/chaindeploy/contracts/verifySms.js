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

import abi from '~/contracts/abi/sms-verification';
import { compiler, source as sourceUrl, output as byteCode } from './code/verifySms';

const isBadge = true;
const id = 'smsverification';
const deployParams = [];
const badgeSource = {
  imageUrl: 'https://raw.githubusercontent.com/paritytech/dapp-assets/1b1beb57ab1f4d3a93a12711b233b5cded791a2f/certifications/sms-verification.svg',
  imageHash: '0x49fa653c35c0a9ce128579883babd673ad4cfc94bf9f1cfe96a2bbc30a7552c6'
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
