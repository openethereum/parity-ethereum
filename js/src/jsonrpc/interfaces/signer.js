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

import { Quantity, Data } from '../types';
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
    params: [{
      type: String,
      desc: 'Domain for which the token is valid. Only requests to this domain will be allowed.',
      example: 'https://parity.io'
    }],
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
          condition: {
            type: Object,
            desc: 'Condition for scheduled transaction. Can be either an integer block number `{ block: 1 }` or UTC timestamp (in seconds) `{ timestamp: 1491290692 }`.',
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
    desc: 'Confirm specific request with rolling token.',
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
          condition: {
            type: Object,
            desc: 'Conditional submission of the transaction. Can be either an integer block number `{ block: 1 }` or UTC timestamp (in seconds) `{ time: 1491290692 }` or `null`.',
            optional: true
          }
        },
        example: {}
      },
      {
        type: String,
        desc: 'Password (initially) or a token returned by the previous call.',
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
          desc: 'Token used to authenticate the next request.'
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
  },

  // Pub-Sub
  subscribePending: {
    desc: `
Starts a subscription for transactions in the confirmation queue.
Each event contains all transactions currently in the queue.

An example notification received by subscribing to this event:
\`\`\`
{"jsonrpc":"2.0","method":"signer_pending","params":{"subscription":"0x416d77337e24399d","result":[]}}
\`\`\`

You can unsubscribe using \`signer_unsubscribePending\` RPC method. Subscriptions are also tied to a transport
connection, disconnecting causes all subscriptions to be canceled.
    `,
    params: [],
    returns: {
      type: String,
      desc: 'Assigned subscription ID',
      example: '0x416d77337e24399d'
    }
  },
  unsubscribePending: {
    desc: 'Unsubscribes from pending transactions subscription.',
    params: [{
      type: String,
      desc: 'Subscription ID',
      example: '0x416d77337e24399d'
    }],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful',
      example: true
    }
  }
};
