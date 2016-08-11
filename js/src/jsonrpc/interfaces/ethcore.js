import { Address, Data, Quantity } from '../types';

export default {
  acceptNonReservedPeers: {
    desc: '?',
    params: [],
    returns: {
      type: Boolean,
      desc: '?'
    }
  },

  addReservedPeer: {
    desc: '?',
    params: [
      {
        type: String,
        desc: 'Enode'
      }
    ],
    returns: {
      type: Boolean,
      desc: '?'
    }
  },

  defaultExtraData: {
    desc: 'Returns the default extra data',
    params: [],
    returns: {
      type: Data,
      desc: 'Extra data'
    }
  },

  devLogs: {
    desc: 'Returns latest logs of your node',
    params: [],
    returns: {
      type: Array,
      desc: 'Development logs'
    }
  },

  devLogsLevels: {
    desc: 'Returns current log level settings',
    params: [],
    returns: {
      type: String,
      decs: 'Current log level'
    }
  },

  dropNonReservedPeers: {
    desc: '?',
    params: [],
    returns: {
      type: Boolean,
      desc: '?'
    }
  },

  extraData: {
    desc: 'Returns currently set extra data',
    params: [],
    returns: {
      type: Data,
      desc: 'Extra data'
    }
  },

  gasFloorTarget: {
    desc: 'Returns current target for gas floor',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Gas Floor Target',
      format: 'outputBigNumberFormatter'
    }
  },

  generateSecretPhrase: {
    desc: 'Creates a secret phrase that can be associated with an account',
    params: [],
    returns: {
      type: String,
      desc: 'The secret phrase'
    }
  },

  minGasPrice: {
    desc: 'Returns currently set minimal gas price',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Minimal Gas Price',
      format: 'outputBigNumberFormatter'
    }
  },

  netChain: {
    desc: 'Returns the name of the connected chain.',
    params: [],
    returns: {
      type: String,
      desc: 'chain name'
    }
  },

  netMaxPeers: {
    desc: 'Returns maximal number of peers.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Maximal number of peers'
    }
  },

  netPort: {
    desc: 'Returns network port the node is listening on.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Port Number'
    }
  },

  nodeName: {
    desc: 'Returns node name (identity)',
    params: [],
    returns: {
      type: String,
      desc: 'Node name'
    }
  },

  phraseToAddress: {
    desc: 'Converts a secret phrase into the corresponting address',
    params: [
      {
        type: String,
        desc: 'The secret'
      }
    ],
    returns: {
      type: Address,
      desc: 'Corresponding address'
    }
  },

  removeReservedPeer: {
    desc: '?',
    params: [
      {
        type: String,
        desc: 'Encode'
      }
    ],
    returns: {
      type: Boolean,
      desc: '?'
    }
  },

  rpcSettings: {
    desc: 'Returns basic settings of rpc (enabled, port, interface).',
    params: [],
    returns: {
      type: Object,
      desc: 'JSON object containing rpc settings'
    }
  },

  setAuthor: {
    desc: 'Changes author (coinbase) for mined blocks.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - Address',
        format: 'inputAddressFormatter'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  setExtraData: {
    desc: 'Changes extra data for newly mined blocks',
    params: [
      {
        type: Data,
        desc: 'Extra Data',
        format: 'utils.toHex'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  setGasFloorTarget: {
    desc: 'Changes current gas floor target.',
    params: [
      {
        type: Quantity,
        desc: 'Gas Floor Target',
        format: 'utils.toHex'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  setMinGasPrice: {
    desc: 'Changes minimal gas price for transaction to be accepted to the queue.',
    params: [
      {
        type: Quantity,
        desc: 'Minimal gas price',
        format: 'utils.toHex'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  setTransactionsLimit: {
    desc: 'Changes limit for transactions in queue.',
    params: [
      {
        type: Quantity,
        desc: 'New Limit',
        format: 'utils.toHex'
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  transactionsLimit: {
    desc: 'Changes limit for transactions in queue.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Current max number of transactions in queue',
      format: 'outputBigNumberFormatter'
    }
  },

  unsignedTransactionsCount: {
    desc: 'Returns number of unsigned transactions when running with Trusted Signer. Error otherwise',
    params: [],
    returns: {
      type: Quantity,
      desc: 'Number of unsigned transactions'
    }
  }
};
