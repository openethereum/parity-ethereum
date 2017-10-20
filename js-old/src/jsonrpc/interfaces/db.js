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

export default {
  getHex: {
    nodoc: 'Not present in Rust code',
    desc: 'Returns binary data from the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      }
    ],
    returns: {
      type: Data,
      desc: 'The previously stored data'
    },
    deprecated: true
  },

  getString: {
    nodoc: 'Not present in Rust code',
    desc: 'Returns string from the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      }
    ],
    returns: {
      type: String,
      desc: 'The previously stored string'
    },
    deprecated: true
  },

  putHex: {
    nodoc: 'Not present in Rust code',
    desc: 'Stores binary data in the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      },
      {
        type: Data,
        desc: 'The data to store'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the value was stored, otherwise `false`'
    },
    deprecated: true
  },

  putString: {
    nodoc: 'Not present in Rust code',
    desc: 'Stores a string in the local database.',
    params: [
      {
        type: String,
        desc: 'Database name'
      },
      {
        type: String,
        desc: 'Key name'
      },
      {
        type: String,
        desc: 'The string to store'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the value was stored, otherwise `false`'
    },
    deprecated: true
  }
};
