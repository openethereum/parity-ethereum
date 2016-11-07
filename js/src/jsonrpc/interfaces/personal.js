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

import { Address, Data, Quantity } from '../types';

export default {
  accountsInfo: {
    desc: 'returns a map of accounts as an object',
    params: [],
    returns: {
      type: Array,
      desc: 'Account metadata',
      details: {
        name: {
          type: String,
          desc: 'Account name'
        },
        meta: {
          type: String,
          desc: 'Encoded JSON string the defines additional account metadata'
        },
        uuid: {
          type: String,
          desc: 'The account UUID, or null if not available/unknown/not applicable.'
        }
      }
    }
  },

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

  listAccounts: {
    desc: 'Returns a list of addresses owned by client.',
    params: [],
    returns: {
      type: Array,
      desc: '20 Bytes addresses owned by the client.'
    }
  },

  listGethAccounts: {
    desc: 'Returns a list of the accounts available from Geth',
    params: [],
    returns: {
      type: Array,
      desc: '20 Bytes addresses owned by the client.'
    }
  },

  importGethAccounts: {
    desc: 'Imports a list of accounts from geth',
    params: [
      {
        type: Array,
        desc: 'List of the geth addresses to import'
      }
    ],
    returns: {
      type: Array,
      desc: 'Array of the imported addresses'
    }
  },

  newAccount: {
    desc: 'Creates new account',
    params: [
      {
        type: String,
        desc: 'Password'
      }
    ],
    returns: {
      type: Address,
      desc: 'The created address'
    }
  },

  newAccountFromPhrase: {
    desc: 'Creates a new account from a recovery passphrase',
    params: [
      {
        type: String,
        desc: 'Phrase'
      },
      {
        type: String,
        desc: 'Password'
      }
    ],
    returns: {
      type: Address,
      desc: 'The created address'
    }
  },

  newAccountFromSecret: {
    desc: 'Creates a new account from a private ethstore secret key',
    params: [
      {
        type: Data,
        desc: 'Secret, 32-byte hex'
      },
      {
        type: String,
        desc: 'Password'
      }
    ],
    returns: {
      type: Address,
      desc: 'The created address'
    }
  },

  newAccountFromWallet: {
    desc: 'Creates a new account from a JSON import',
    params: [
      {
        type: String,
        desc: 'JSON'
      },
      {
        type: String,
        desc: 'Password'
      }
    ],
    returns: {
      type: Address,
      desc: 'The created address'
    }
  },

  setAccountName: {
    desc: 'Sets a name for the account',
    params: [
      {
        type: Address,
        desc: 'Address'
      },
      {
        type: String,
        desc: 'Name'
      }
    ],
    returns: {
      type: Object,
      desc: 'Returns null in all cases'
    }
  },

  setAccountMeta: {
    desc: 'Sets metadata for the account',
    params: [
      {
        type: Address,
        desc: 'Address'
      },
      {
        type: String,
        desc: 'Metadata (JSON encoded)'
      }
    ],
    returns: {
      type: Object,
      desc: 'Returns null in all cases'
    }
  },

  signAndSendTransaction: {
    desc: 'Sends and signs a transaction given account passphrase. Does not require the account to be unlocked nor unlocks the account for future transactions. ',
    params: [
      {
        type: Object,
        desc: 'The transaction object',
        details: {
          from: {
            type: Address,
            desc: '20 Bytes - The address the transaction is send from'
          },
          to: {
            type: Address,
            desc: '20 Bytes - (optional when creating new contract) The address the transaction is directed to'
          },
          gas: {
            type: Quantity,
            desc: 'Integer of the gas provided for the transaction execution. It will return unused gas',
            optional: true,
            default: 90000
          },
          gasPrice: {
            type: Quantity,
            desc: 'Integer of the gasPrice used for each paid gas',
            optional: true,
            default: 'To-Be-Determined'
          },
          value: {
            type: Quantity,
            desc: 'Integer of the value send with this transaction',
            optional: true
          },
          data: {
            type: Data,
            desc: 'The compiled code of a contract OR the hash of the invoked method signature and encoded parameters. For details see [Ethereum Contract ABI](https://github.com/ethereum/wiki/wiki/Ethereum-Contract-ABI)'
          },
          nonce: {
            type: Quantity,
            desc: 'Integer of a nonce. This allows to overwrite your own pending transactions that use the same nonce.',
            optional: true
          }
        }
      },
      {
        type: String,
        desc: 'Passphrase to unlock `from` account.'
      }
    ],
    returns: {
      type: Data,
      desc: '32 Bytes - the transaction hash, or the zero hash if the transaction is not yet available'
    }
  },

  signerEnabled: {
    desc: 'Returns whether signer is enabled/disabled.',
    params: [],
    returns: {
      type: Boolean,
      desc: 'true when enabled, false when disabled'
    }
  },

  unlockAccount: {
    desc: '?',
    params: [
      '?', '?', '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  }
};
