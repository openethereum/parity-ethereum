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

import { padAddress } from './pad';

export function sliceData (_data) {
  if (!_data || !_data.length) {
    return [];
  }

  let data = (_data.substr(0, 2) === '0x') ? _data.substr(2) : _data;

  if (!data.length) {
    data = padAddress('');
  }

  return data.match(/.{1,64}/g);
}
