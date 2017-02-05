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

import { Quantity, Data, BlockNumber } from '../types';
import { fromDecimal, Dummy } from '../helpers';

export default {
  generateAuthorizationToken: {
    desc: 'Generates a new authorization token.',
    params: [],
    returns: {
      type: String,
      desc: 'The new authorization token.',
      example: 'bNGY-iIPB-j7zK-RSYZ'
    }
  },

  generateWebProxyAccessToken: {
    desc: 'Generates a new web proxy access token.',
    params: [],
    returns: {
      type: String,
      desc: 'The new web proxy access token.',
      example: 'MOWm0tEJjwthDiTU'
    }
  },

  requestsToConfirm: {
    desc: 'Returns a list of the transactions awaiting authorization.',
    params: [],
    returns: {
      // TODO: Types of the fields of transaction objects? Link to a transaction object in another page?
      type: Array,
      desc: 'A list of the outstanding transactions.',
      example: new Dummy('[ ... ]')
    }
  },

  confirmRequest: {
    desc: 'Confirm a request in the signer queue',
    params: [
      {
        type: Quantity,
        desc: 'The request id.',
        example: fromDecimal(1)
      },
      {
        type: Object,
        desc: 'Modify the transaction before confirmation.',
        details: {
          gasPrice: {
            type: Quantity,
            desc: 'Modify the gas price provided by the sender in Wei.',
            optional: true
          },
          gas: {
            type: Quantity,
            desc: 'Gas provided by the sender in Wei.',
            optional: true
          },
          minBlock: {
            type: BlockNumber,
            desc: 'Integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`. Request will not be propagated till the given block is reached.',
            optional: true
          }
        },
        example: {}
      },
      {
        type: String,
        desc: 'The account password',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Object,
      desc: 'The status of the confirmation, depending on the request type.',
      example: {}
    }
  },

  confirmRequestRaw: {
    desc: 'Confirm a request in the signer queue providing signed request.',
    params: [
      {
        type: Quantity,
        desc: 'Integer - The request id',
        example: fromDecimal(1)
      },
      {
        type: Data,
        desc: 'Signed request (RLP encoded transaction)',
        example: '0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675'
      }
    ],
    returns: {
      type: Object,
      desc: 'The status of the confirmation, depending on the request type.',
      example: {}
    }
  },

  confirmRequestWithToken: {
    desc: 'Confirm specific request with token.',
    params: [
      {
        type: Quantity,
        desc: 'The request id.',
        example: fromDecimal(1)
      },
      {
        type: Object,
        desc: 'Modify the transaction before confirmation.',
        details: {
          gasPrice: {
            type: Quantity,
            desc: 'Modify the gas price provided by the sender in Wei.',
            optional: true
          },
          gas: {
            type: Quantity,
            desc: 'Gas provided by the sender in Wei.',
            optional: true
          },
          minBlock: {
            type: BlockNumber,
            desc: 'Integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`. Request will not be propagated till the given block is reached.',
            optional: true
          }
        },
        example: {}
      },
      {
        type: String,
        desc: 'Password.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Object,
      desc: 'Status.',
      details: {
        result: {
          type: Object,
          desc: 'The status of the confirmation, depending on the request type.'
        },
        token: {
          type: String,
          desc: 'Token used to authenticate the request.'
        }
      },
      example: {
        result: new Dummy('{ ... }'),
        token: 'cAF2w5LE7XUZ3v3N'
      }
    }
  },

  rejectRequest: {
    desc: 'Rejects a request in the signer queue',
    params: [
      {
        type: Quantity,
        desc: 'Integer - The request id',
        example: fromDecimal(1)
      }
    ],
    returns: {
      type: Boolean,
      desc: 'The status of the rejection',
      example: true
    }
  },

  signerEnabled: {
    nodoc: 'Not present in Rust code',
    desc: 'Returns whether signer is enabled/disabled.',
    params: [],
    returns: {
      type: Boolean,
      desc: '`true` when enabled, `false` when disabled.',
      example: true
    }
  }
};
