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

import { Quantity } from '../types';
import { fromDecimal } from '../helpers';

export default {
  listening: {
    desc: 'Returns `true` if client is actively listening for network connections.',
    params: [],
    returns: {
      type: Boolean,
      desc: '`true` when listening, otherwise `false`.',
      example: true
    }
  },

  peerCount: {
    desc: 'Returns number of peers currenly connected to the client.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Integer of the number of connected peers',
      format: 'utils.toDecimal',
      example: fromDecimal(2)
    }
  },

  version: {
    desc: 'Returns the current network protocol version.',
    params: [],
    returns: {
      type: String,
      desc: 'The current network protocol version',
      example: '8995'
    }
  }
};
