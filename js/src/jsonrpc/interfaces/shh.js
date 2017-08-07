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

import { Data, Quantity, Float } from '../types';

export default {
  info: {
    desc: 'Returns the current whisper protocol version.',
    params: [],
    returns: {
      type: Object,
      desc: 'The current whisper protocol version',
      details: {
        minPow: {
          type: Float,
          desc: 'required PoW threshold for a message to be accepted into the local pool, or null if there is empty space in the pool.'
        },
        messages: {
          type: Quantity,
          desc: 'Number of messages in the pool.'
        },
        memory: {
          type: Quantity,
          desc: 'Amount of memory used by messages in the pool.'
        },
        targetMemory: {
          type: Quantity,
          desc: 'Target amount of memory for the pool.'
        }
      }
    }
  },

  post: {
    desc: 'Sends a whisper message.',
    params: [
      {
        type: Object, desc: 'The whisper post object:', format: 'inputPostFormatter',
        details: {
          from: {
            type: Data, desc: '60 Bytes - The identity of the sender',
            optional: true
          },
          to: {
            type: Data, desc: '60 Bytes - The identity of the receiver. When present whisper will encrypt the message so that only the receiver can decrypt it',
            optional: true
          },
          topics: {
            type: Array, desc: 'Array of `Data` topics, for the receiver to identify messages'
          },
          payload: {
            type: Data, desc: 'The payload of the message'
          },
          priority: {
            type: Quantity, desc: 'The integer of the priority in a rang from ... (?)'
          },
          ttl: {
            type: Quantity, desc: 'Integer of the time to live in seconds.'
          }
        }
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the message was send, otherwise `false`'
    }
  },

  newKeyPair: {
    desc: 'Generate a new key pair (identity) for asymmetric encryption.',
    params: [],
    returns: {
      type: Data,
      desc: '32 Bytes - the address of the new identiy'
    }
  },

  addPrivateKey: {
    desc: 'Import a private key to use for asymmetric decryption.',
    params: [
      {
        type: Data,
        desc: '32 Bytes - The private key to import'
      }
    ],
    returns: {
      type: Data,
      desc: '`32 Bytes`  A unique identity to refer to this keypair by.'
    }
  },

  newSymKey: {
    desc: 'Generate a key pair(identity) for symmetric encryption.',
    params: [],
    returns: {
      type: Data,
      desc: '32 Bytes - the address of the new identiy'
    }
  },

  getPublicKey: {
    desc: 'Get the public key associated with an asymmetric identity.',
    params: [
      {
        type: Data,
        desc: '32 Bytes - The identity to fetch the public key for.'
      }
    ],
    returns: {
      type: Data,
      desc: '`64 Bytes` - The public key of the asymmetric identity.'
    }
  },

  getPrivateKey: {
    desc: 'Get the private key associated with an asymmetric identity.',
    params: [
      {
        type: Data,
        desc: '32 Bytes - The identity to fetch the private key for.'
      }
    ],
    returns: {
      type: Data,
      desc: '`32 Bytes` - The private key of the asymmetric identity.'
    }
  },

  getSymKey: {
    desc: 'Get the key associated with a symmetric identity.',
    params: [
      {
        type: Data,
        desc: '`32 Bytes` - The identity to fetch the key for.'
      }
    ],
    returns: {
      type: Data,
      desc: '`64 Bytes` - The key of the asymmetric identity.'
    }
  },

  deleteKey: {
    desc: 'Delete the key or key pair denoted by the given identity.',
    params: [
      {
        type: Data,
        desc: '`32 Bytes` - The identity to remove.'
      }
    ],
    returns: {
      type: Data,
      desc: '`true` on successful removal, `false` on unkown identity'
    }
  },

  newMessageFilter: {
    desc: 'Create a new polled filter for messages.',
    params: [
      {
        type: Object, desc: 'The filter options:',
        details: {
          decryptWith: {
            type: Data,
            desc: '`32 bytes` - Identity of key used for description. null if listening for broadcasts.'
          },
          from: {
            type: Data, desc: '`32 Bytes` - if present, only accept messages signed by this key.',
            optional: true
          },
          topics: {
            type: Array,
            desc: 'Array of `Data`. Only accept messages matching these topics. Should be non-empty.'
          }
        }
      }
    ],
    returns: {
      type: Data,
      desc: '`32 bytes` - Unique identity for this filter.'
    }
  },

  getFilterMesssages: {
    nodoc: 'Not present in Rust code',
    desc: 'Uninstalls a filter with given id. Should always be called when watch is no longer needed.\nAdditonally Filters timeout when they aren\'t requested with [shh_getFilterChanges](#shh_getfilterchanges) for a period of time.',
    params: [
      {
        type: Quantity,
        desc: 'The filter id'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the filter was successfully uninstalled, otherwise `false`'
    }
  },

  getFilterMessages: {
    desc: 'Polling method for whisper filters. Returns new messages since the last call of this method.\n**Note** calling the [shh_getMessages](#shh_getmessages) method, will reset the buffer for this method, so that you won\'t receive duplicate messages.',
    params: [
      {
        type: Data,
        desc: '`32 bytes` - Unique identity to fetch changes for.'
      }
    ],
    returns: {
      type: Array,
      desc: 'Array of `messages` received since last poll',
      details: {
        from: {
          type: Data,
          desc: '`64 bytes` - Public key that signed this message or null'
        },
        recipient: {
          type: Data,
          desc: '`32 bytes` - local identity which decrypted this message, or null if broadcast.'
        },
        ttl: {
          type: Quantity,
          desc: 'time to live of the message in seconds.'
        },
        topics: {
          type: Array,
          desc: 'Array of `Data` - Topics which matched the filter'
        },
        timestamp: {
          type: Quantity,
          desc: 'Unix timestamp of the message'
        },
        payload: {
          type: Data,
          desc: 'The message body'
        },
        padding: {
          type: Data,
          desc: 'Optional padding which was decoded.'
        }
      }
    }
  },

  deleteMessageFilter: {
    desc: 'Delete a message filter by identifier',
    params: [
      {
        type: Data,
        desc: '`32 bytes` - The identity of the filter to delete.'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` on deletion, `false` on unrecognized ID.'
    }
  },
  subscribe: {
    desc: 'Open a subscription to a filter.',
    params: [{
      type: Data,
      desc: 'See [shh_newMessageFilter](#shh_newmessagefilter)'
    }],
    returns: {
      type: Quantity,
      desc: 'Unique subscription identifier'
    }
  },
  unsubscribe: {
    desc: 'Close a subscribed filter',
    params: [{
      type: Quantity,
      desc: 'Unique subscription identifier'
    }],
    returns: {
      type: Boolean,
      desc: '`true` on success, `false` on unkown subscription ID.'
    }
  }
};
