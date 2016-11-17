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

import { Quantity, Data } from '../types';

export default {
  generateAuthorizationToken: {
    desc: 'Generates a new authorization token',
    params: [],
    returns: {
      type: String,
      desc: 'The new authorization token'
    }
  },

  requestsToConfirm: {
    desc: 'Returns a list of the transactions requiring authorization',
    params: [],
    returns: {
      type: Array,
      desc: 'A list of the outstanding transactions'
    }
  },

  confirmRequest: {
    desc: 'Confirm a request in the signer queue',
    params: [
      {
        type: Quantity,
        desc: 'The request id'
      },
      {
        type: Object,
        desc: 'The request options'
      },
      {
        type: String,
        desc: 'The account password'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'The status of the confirmation'
    }
  },

  confirmRequestRaw: {
    desc: 'Confirm a request in the signer queue providing signed request.',
    params: [
      {
        type: Quantity,
        desc: 'The request id'
      },
      {
        type: Data,
        desc: 'Signed request (transaction RLP)'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'The status of the confirmation'
    }
  },

  rejectRequest: {
    desc: 'Rejects a request in the signer queue',
    params: [
      {
        type: Quantity,
        desc: 'The request id'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'The status of the rejection'
    }
  },

  signerEnabled: {
    desc: 'Returns whether signer is enabled/disabled.',
    params: [],
    returns: {
      type: Boolean,
      desc: 'true when enabled, false when disabled'
    }
  }
};
