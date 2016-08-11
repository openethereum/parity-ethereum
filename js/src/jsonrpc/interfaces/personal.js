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

  listAccounts: {
    desc: 'Returns a list of addresses owned by client.',
    params: [],
    returns: {
      type: Array,
      desc: '20 Bytes addresses owned by the client.'
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
    desc: 'Creates a new account from a brainwallet passphrase',
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
