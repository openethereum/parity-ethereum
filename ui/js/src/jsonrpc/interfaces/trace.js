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

import { BlockNumber, Hash, Integer } from '../types';

export default {
  filter: {
    desc: 'Returns traces matching given filter',
    params: [
      {
        type: Object,
        desc: 'The filter object'
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces matching given filter'
    }
  },

  get: {
    desc: 'Returns trace at given position.',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash'
      },
      {
        type: Integer,
        desc: 'Trace position witing transaction'
      }
    ],
    returns: {
      type: Object,
      desc: 'Trace object'
    }
  },

  transaction: {
    desc: 'Returns all traces of given transaction',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash'
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces of given transaction'
    }
  },

  block: {
    desc: 'Returns traces created at given block',
    params: [
      {
        type: BlockNumber,
        desc: 'Integer block number, or \'latest\' for the last mined block or \'pending\', \'earliest\' for not yet mined transactions'
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces'
    }
  }
};
