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

import { Data } from '../types';
import { withComment } from '../helpers';

export default {
  clientVersion: {
    desc: 'Returns the current client version.',
    params: [],
    returns: {
      type: String,
      desc: 'The current client version',
      example: 'Parity//v1.5.0-unstable-9db3f38-20170103/x86_64-linux-gnu/rustc1.14.0'
    }
  },

  sha3: {
    desc: 'Returns Keccak-256 (**not** the standardized SHA3-256) of the given data.',
    params: [
      {
        type: String,
        desc: 'The data to convert into a SHA3 hash.',
        example: withComment('0x68656c6c6f20776f726c64', '"hello world"')
      }
    ],
    returns: {
      type: Data,
      desc: 'The Keccak-256 hash of the given string.',
      example: '0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad'
    }
  }
};
