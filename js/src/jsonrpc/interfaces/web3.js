// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { Data } from '../types';

export default {
  clientVersion: {
    desc: 'Returns the current client version.',
    params: [],
    returns: {
      type: String,
      desc: 'The current client version'
    }
  },

  sha3: {
    desc: 'Returns Keccak-256 (*not* the standardized SHA3-256) of the given data.',
    params: [
      {
        type: String,
        desc: 'The data to convert into a SHA3 hash'
      }
    ],
    returns: {
      type: Data,
      desc: 'The SHA3 result of the given string'
    }
  }
};
