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

import { Address, BlockNumber, Data, Hash, Quantity, CallRequest, TransactionRequest } from '../types';
import { withPreamble, fromDecimal, withComment, Dummy } from '../helpers';

const SUBDOC_PUBSUB = 'pubsub';

export default withPreamble(`

## The default block parameter

The following methods have an optional extra \`defaultBlock\` parameter:

- [eth_estimateGas](#eth_estimategas)
- [eth_getBalance](#eth_getbalance)
- [eth_getCode](#eth_getcode)
- [eth_getTransactionCount](#eth_gettransactioncount)
- [eth_getStorageAt](#eth_getstorageat)
- [eth_call](#eth_call)

When requests are made that act on the state of Ethereum, the last parameter determines the height of the block.

The following options are possible for the \`defaultBlock\` parameter:

- \`Quantity\`/\`Integer\` - an integer block number;
- \`String "earliest"\` - for the earliest/genesis block;
- \`String "latest"\` - for the latest mined block;
- \`String "pending"\` - for the pending state/transactions.

`, {
  accounts: {
    desc: 'Returns a list of addresses owned by client.',
    params: [],
    returns: {
      type: Array,
      desc: '20 Bytes - addresses owned by the client.',
      example: ['0x407d73d8a49eeb85d32cf465507dd71d507100c1']
    }
  },

  blockNumber: {
    desc: 'Returns the number of most recent block.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'integer of the current block number the client is on.',
      example: fromDecimal(1207)
    }
  },

  call: {
    desc: 'Executes a new message call immediately without creating a transaction on the block chain.',
    params: [
      {
        type: CallRequest,
        desc: 'The transaction call object.',
        format: 'inputCallFormatter',
        example: {
          from: '0x407d73d8a49eeb85d32cf465507dd71d507100c1',
          to: '0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b',
          value: fromDecimal(100000)
        }
      },
      {
        type: BlockNumber,
        desc: 'Integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`, see the [default block parameter](#the-default-block-parameter).',
        format: 'inputDefaultBlockNumberFormatter',
        optional: true
      }
    ],
    returns: {
      type: Data,
      desc: 'the return value of executed contract.',
      example: '0x'
    }
  },

  coinbase: {
    desc: 'Returns the client coinbase address.',
    params: [],
    returns: {
      type: Address,
      desc: 'The current coinbase address.',
      example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
    }
  },

  estimateGas: {
    desc: 'Makes a call or transaction, which won\'t be added to the blockchain and returns the used gas, which can be used for estimating the used gas.',
    params: [
      {
        type: CallRequest,
        desc: 'Same as [eth_call](#eth_call) parameters, except that all properties are optional.',
        format: 'inputCallFormatter',
        example: new Dummy('{ ... }')
      },
      {
        type: BlockNumber,
        desc: 'Integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`, see the [default block parameter](#the-default-block-parameter).',
        format: 'inputDefaultBlockNumberFormatter',
        optional: true
      }
    ],
    returns: {
      type: Quantity,
      desc: 'The amount of gas used.',
      format: 'utils.toDecimal',
      example: fromDecimal(21000)
    }
  },

  fetchQueuedTransactions: {
    nodoc: 'Not present in Rust code',
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  flush: {
    nodoc: 'Not present in Rust code',
    desc: '?',
    params: [],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  gasPrice: {
    desc: 'Returns the current price per gas in wei.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'integer of the current gas price in wei.',
      example: fromDecimal(10000000000000)
    }
  },

  getBalance: {
    desc: 'Returns the balance of the account of given address.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - address to check for balance.',
        format: 'inputAddressFormatter',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: BlockNumber,
        desc: 'integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`, see the [default block parameter](#the-default-block-parameter).',
        format: 'inputDefaultBlockNumberFormatter',
        optional: true
      }
    ],
    returns: {
      type: Quantity,
      desc: 'integer of the current balance in wei.',
      format: 'outputBigNumberFormatter',
      example: '0x0234c8a3397aab58'
    }
  },

  getBlockByHash: {
    desc: 'Returns information about a block by hash.',
    params: [
      {
        type: Hash,
        desc: 'Hash of a block.',
        example: '0xe670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331'
      },
      {
        type: Boolean,
        desc: 'If `true` it returns the full transaction objects, if `false` only the hashes of the transactions.',
        example: true
      }
    ],
    returns: {
      type: Object,
      desc: 'A block object, or `null` when no block was found.',
      details: {
        number: {
          type: Quantity,
          desc: 'The block number. `null` when its pending block'
        },
        hash: {
          type: Hash,
          desc: '32 Bytes - hash of the block. `null` when its pending block'
        },
        parentHash: {
          type: Hash,
          desc: '32 Bytes - hash of the parent block'
        },
        nonce: {
          type: Data,
          desc: '8 Bytes - hash of the generated proof-of-work. `null` when its pending block'
        },
        sha3Uncles: {
          type: Data,
          desc: '32 Bytes - SHA3 of the uncles data in the block'
        },
        logsBloom: {
          type: Data,
          desc: '256 Bytes - the bloom filter for the logs of the block. `null` when its pending block'
        },
        transactionsRoot: {
          type: Data,
          desc: '32 Bytes - the root of the transaction trie of the block'
        },
        stateRoot: {
          type: Data,
          desc: '32 Bytes - the root of the final state trie of the block'
        },
        receiptsRoot: {
          type: Data, desc: '32 Bytes - the root of the receipts trie of the block'
        },
        miner: {
          type: Address,
          desc: '20 Bytes - the address of the beneficiary to whom the mining rewards were given'
        },
        difficulty: {
          type: Quantity,
          desc: 'integer of the difficulty for this block'
        },
        totalDifficulty: {
          type: Quantity,
          desc: 'integer of the total difficulty of the chain until this block'
        },
        extraData: {
          type: Data,
          desc: 'the \'extra data\' field of this block'
        },
        size: {
          type: Quantity,
          desc: 'integer the size of this block in bytes'
        },
        gasLimit: {
          type: Quantity,
          desc: 'the maximum gas allowed in this block'
        },
        gasUsed: {
          type: Quantity,
          desc: 'the total used gas by all transactions in this block'
        },
        timestamp: {
          type: Quantity,
          desc: 'the unix timestamp for when the block was collated'
        },
        transactions: {
          type: Array,
          desc: 'Array of transaction objects, or 32 Bytes transaction hashes depending on the last given parameter'
        },
        uncles: {
          type: Array,
          desc: 'Array of uncle hashes'
        }
      },
      example: {
        number: fromDecimal(436),
        hash: '0xe670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331',
        parentHash: '0x9646252be9520f6e71339a8df9c55e4d7619deeb018d2a3f2d21fc165dde5eb5',
        sealFields: ['0xe04d296d2460cfb8472af2c5fd05b5a214109c25688d3704aed5484f9a7792f2', '0x0000000000000042'],
        sha3Uncles: '0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347',
        logsBloom: '0xe670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331',
        transactionsRoot: '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
        stateRoot: '0xd5855eb08b3387c0af375e9cdb6acfc05eb8f519e419b874b6ff2ffda7ed1dff',
        miner: '0x4e65fda2159562a496f9f3522f89122a3088497a',
        difficulty: fromDecimal(163591),
        totalDifficulty: fromDecimal(163591),
        extraData: '0x0000000000000000000000000000000000000000000000000000000000000000',
        size: fromDecimal(163591),
        gasLimit: fromDecimal(653145),
        minGasPrice: fromDecimal(653145),
        gasUsed: fromDecimal(653145),
        timestamp: fromDecimal(1424182926),
        transactions: [new Dummy('{ ... }, { ... }, ...')],
        uncles: ['0x1606e5...', '0xd5145a9...']
      }
    }
  },

  getBlockByNumber: {
    desc: 'Returns information about a block by block number.',
    params: [
      {
        type: BlockNumber,
        desc: 'integer of a block number, or the string `\'earliest\'`, `\'latest\'` or `\'pending\'`, as in the [default block parameter](#the-default-block-parameter).',
        example: fromDecimal(436)
      },
      {
        type: Boolean,
        desc: 'If `true` it returns the full transaction objects, if `false` only the hashes of the transactions.',
        example: true
      }
    ],
    returns: 'See [eth_getBlockByHash](#eth_getblockbyhash)'
  },

  getBlockTransactionCountByHash: {
    desc: 'Returns the number of transactions in a block from a block matching the given block hash.',
    params: [
      {
        type: Hash,
        desc: '32 Bytes - hash of a block.',
        example: '0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238'
      }
    ],
    returns: {
      type: Quantity,
      desc: 'integer of the number of transactions in this block.',
      example: fromDecimal(11)
    }
  },

  getBlockTransactionCountByNumber: {
    desc: 'Returns the number of transactions in a block from a block matching the given block number.',
    params: [
      {
        type: BlockNumber,
        desc: 'integer of a block number, or the string `\'earliest\'`, `\'latest\'` or `\'pending\'`, as in the [default block parameter](#the-default-block-parameter).',
        example: fromDecimal(232)
      }
    ],
    returns: {
      type: Quantity,
      desc: 'integer of the number of transactions in this block.',
      example: fromDecimal(10)
    }
  },

  getCode: {
    desc: 'Returns code at a given address.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - address.',
        format: 'inputAddressFormatter',
        example: '0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b'
      },
      {
        type: BlockNumber,
        desc: 'integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`, see the [default block parameter](#the-default-block-parameter).',
        format: 'inputDefaultBlockNumberFormatter',
        example: fromDecimal(2)
      }
    ],
    returns: {
      type: Data,
      desc: 'the code from the given address.',
      example: '0x600160008035811a818181146012578301005b601b6001356025565b8060005260206000f25b600060078202905091905056'
    }
  },

  getFilterChanges: {
    desc: 'Polling method for a filter, which returns an array of logs which occurred since last poll.',
    params: [
      {
        type: Quantity,
        desc: 'The filter id.',
        example: fromDecimal(22)
      }
    ],
    returns: {
      type: Array,
      desc: 'Array of log objects, or an empty array if nothing has changed since last poll.',
      example: [
        {
          logIndex: fromDecimal(1),
          blockNumber: fromDecimal(436),
          blockHash: '0x8216c5785ac562ff41e2dcfdf5785ac562ff41e2dcfdf829c5a142f1fccd7d',
          transactionHash: '0xdf829c5a142f1fccd7d8216c5785ac562ff41e2dcfdf5785ac562ff41e2dcf',
          transactionIndex: fromDecimal(0),
          address: '0x16c5785ac562ff41e2dcfdf829c5a142f1fccd7d',
          data: '0x0000000000000000000000000000000000000000000000000000000000000000',
          topics: ['0x59ebeb90bc63057b6515673c3ecf9438e5058bca0f92585014eced636878c9a5']
        },
        new Dummy('...')
      ]
    }
  },

  getFilterChangesEx: {
    nodoc: 'Not present in Rust code', // https://github.com/ethereum/wiki/wiki/Proposal:-Reversion-Notification
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  getFilterLogs: {
    desc: 'Returns an array of all logs matching filter with given id.',
    params: [
      {
        type: Quantity,
        desc: 'The filter id.',
        example: fromDecimal(22)
      }
    ],
    returns: 'See [eth_getFilterChanges](#eth_getfilterchanges)'
  },

  getFilterLogsEx: {
    nodoc: 'Not present in Rust code', // https://github.com/ethereum/wiki/wiki/Proposal:-Reversion-Notification
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  getLogs: {
    desc: 'Returns an array of all logs matching a given filter object.',
    params: [
      {
        type: Object,
        desc: 'The filter object, see [eth_newFilter parameters](#eth_newfilter).',
        example: {
          topics: ['0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b']
        }
      }
    ],
    returns: 'See [eth_getFilterChanges](#eth_getfilterchanges)'
  },

  getLogsEx: {
    nodoc: 'Not present in Rust code', // https://github.com/ethereum/wiki/wiki/Proposal:-Reversion-Notification
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  getStorageAt: {
    desc: 'Returns the value from a storage position at a given address.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - address of the storage.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: Quantity,
        desc: 'integer of the position in the storage.',
        format: 'utils.toHex',
        example: fromDecimal(0)
      },
      {
        type: BlockNumber,
        desc: 'integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`, see the [default block parameter](#the-default-block-parameter).',
        format: 'inputDefaultBlockNumberFormatter',
        example: fromDecimal(2),
        optional: true
      }
    ],
    returns: {
      type: Data,
      desc: 'the value at this storage position.',
      example: '0x0000000000000000000000000000000000000000000000000000000000000003'
    }
  },

  getTransactionByHash: {
    desc: 'Returns the information about a transaction requested by transaction hash.',
    params: [
      {
        type: Hash,
        desc: '32 Bytes - hash of a transaction.',
        example: '0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238'
      }
    ],
    returns: {
      type: Object,
      desc: 'A transaction object, or `null` when no transaction was found:',
      format: 'outputTransactionFormatter',
      details: {
        hash: {
          type: Hash,
          desc: '32 Bytes - hash of the transaction.'
        },
        nonce: {
          type: Quantity,
          desc: 'the number of transactions made by the sender prior to this one.'
        },
        blockHash: {
          type: Hash,
          desc: '32 Bytes - hash of the block where this transaction was in. `null` when its pending.'
        },
        blockNumber: {
          type: BlockNumber,
          desc: 'block number where this transaction was in. `null` when its pending.'
        },
        transactionIndex: {
          type: Quantity,
          desc: 'integer of the transactions index position in the block. `null` when its pending.'
        },
        from: {
          type: Address,
          desc: '20 Bytes - address of the sender.'
        },
        to: {
          type: Address,
          desc: '20 Bytes - address of the receiver. `null` when its a contract creation transaction.'
        },
        value: {
          type: Quantity,
          desc: 'value transferred in Wei.'
        },
        gasPrice: {
          type: Quantity,
          desc: 'gas price provided by the sender in Wei.'
        },
        gas: {
          type: Quantity,
          desc: 'gas provided by the sender.'
        },
        input: {
          type: Data,
          desc: 'the data send along with the transaction.'
        },
        v: {
          type: Quantity,
          desc: 'the standardised V field of the signature.'
        },
        standard_v: {
          type: Quantity,
          desc: 'the standardised V field of the signature (0 or 1).'
        },
        r: {
          type: Quantity,
          desc: 'the R field of the signature.'
        },
        raw: {
          type: Data,
          desc: 'raw transaction data'
        },
        publicKey: {
          type: Hash,
          desc: 'public key of the signer.'
        },
        chainId: {
          type: Quantity,
          desc: 'the chain id of the transaction, if any.'
        },
        creates: {
          type: Hash,
          desc: 'creates contract hash'
        },
        condition: {
          type: Object,
          optional: true,
          desc: 'conditional submission, Block number in `block` or timestamp in `time` or `null`. (parity-feature)'
        }
      },
      example: {
        hash: '0xc6ef2fc5426d6ad6fd9e2a26abeab0aa2411b7ab17f30a99d3cb96aed1d1055b',
        nonce: fromDecimal(0),
        blockHash: '0xbeab0aa2411b7ab17f30a99d3cb9c6ef2fc5426d6ad6fd9e2a26a6aed1d1055b',
        blockNumber: fromDecimal(5599),
        transactionIndex: fromDecimal(1),
        from: '0x407d73d8a49eeb85d32cf465507dd71d507100c1',
        to: '0x853f43d8a49eeb85d32cf465507dd71d507100c1',
        value: fromDecimal(520464),
        gas: fromDecimal(520464),
        gasPrice: '0x09184e72a000',
        input: '0x603880600c6000396000f300603880600c6000396000f3603880600c6000396000f360'
      }
    }
  },

  getTransactionByBlockHashAndIndex: {
    desc: 'Returns information about a transaction by block hash and transaction index position.',
    params: [
      {
        type: Hash,
        desc: 'hash of a block.',
        example: '0xe670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331'
      },
      {
        type: Quantity,
        desc: 'integer of the transaction index position.',
        example: fromDecimal(0)
      }
    ],
    returns: 'See [eth_getBlockByHash](#eth_gettransactionbyhash)'
  },

  getTransactionByBlockNumberAndIndex: {
    desc: 'Returns information about a transaction by block number and transaction index position.',
    params: [
      {
        type: BlockNumber,
        desc: 'a block number, or the string `\'earliest\'`, `\'latest\'` or `\'pending\'`, as in the [default block parameter](#the-default-block-parameter).',
        example: fromDecimal(668)
      },
      {
        type: Quantity,
        desc: 'The transaction index position.',
        example: fromDecimal(0)
      }
    ],
    returns: 'See [eth_getBlockByHash](#eth_gettransactionbyhash)'
  },

  getTransactionCount: {
    desc: 'Returns the number of transactions *sent* from an address.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - address.',
        example: '0x407d73d8a49eeb85d32cf465507dd71d507100c1'
      },
      {
        type: BlockNumber,
        desc: 'integer block number, or the string `\'latest\'`, `\'earliest\'` or `\'pending\'`, see the [default block parameter](#the-default-block-parameter).',
        format: 'inputDefaultBlockNumberFormatter',
        optional: true
      }
    ],
    returns: {
      type: Quantity,
      desc: 'integer of the number of transactions send from this address.',
      format: 'utils.toDecimal',
      example: fromDecimal(1)
    }
  },

  getTransactionReceipt: {
    desc: 'Returns the receipt of a transaction by transaction hash.\n\n**Note** That the receipt is available even for pending transactions.',
    params: [
      {
        type: Hash,
        desc: 'hash of a transaction.',
        example: '0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238'
      }
    ],
    returns: {
      type: Object,
      desc: 'A transaction receipt object, or `null` when no receipt was found:',
      format: 'outputTransactionReceiptFormatter',
      details: {
        transactionHash: {
          type: Hash,
          desc: '32 Bytes - hash of the transaction.'
        },
        transactionIndex: {
          type: Quantity,
          desc: 'integer of the transactions index position in the block.'
        },
        blockHash: {
          type: Hash,
          desc: '32 Bytes - hash of the block where this transaction was in.'
        },
        blockNumber: {
          type: BlockNumber,
          desc: 'block number where this transaction was in.'
        },
        cumulativeGasUsed: {
          type: Quantity,
          desc: 'The total amount of gas used when this transaction was executed in the block.'
        },
        gasUsed: {
          type: Quantity,
          desc: 'The amount of gas used by this specific transaction alone.'
        },
        contractAddress: {
          type: Address,
          desc: '20 Bytes - The contract address created, if the transaction was a contract creation, otherwise `null`.'
        },
        logs: {
          type: Array,
          desc: 'Array of log objects, which this transaction generated.'
        }
      },
      example: {
        transactionHash: '0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238',
        transactionIndex: fromDecimal(1),
        blockNumber: fromDecimal(11),
        blockHash: '0xc6ef2fc5426d6ad6fd9e2a26abeab0aa2411b7ab17f30a99d3cb96aed1d1055b',
        cumulativeGasUsed: fromDecimal(13244),
        gasUsed: fromDecimal(1244),
        contractAddress: withComment('0xb60e8dd61c5d32be8058bb8eb970870f07233155', 'or null, if none was created'),
        logs: withComment([new Dummy('{ ... }, { ... }, ...]')], 'logs as returned by eth_getFilterLogs, etc.')
      }
    }
  },

  getUncleByBlockHashAndIndex: {
    desc: 'Returns information about a uncle of a block by hash and uncle index position.\n\n**Note:** An uncle doesn\'t contain individual transactions.',
    params: [
      {
        type: Hash,
        desc: 'Hash of a block.',
        example: '0xc6ef2fc5426d6ad6fd9e2a26abeab0aa2411b7ab17f30a99d3cb96aed1d1055b'
      },
      {
        type: Quantity,
        desc: 'The uncle\'s index position.',
        example: fromDecimal(0)
      }
    ],
    returns: 'See [eth_getBlockByHash](#eth_getblockbyhash)'
  },

  getUncleByBlockNumberAndIndex: {
    desc: 'Returns information about a uncle of a block by number and uncle index position.\n\n**Note:** An uncle doesn\'t contain individual transactions.',
    params: [
      {
        type: BlockNumber,
        desc: 'a block number, or the string `\'earliest\'`, `\'latest\'` or `\'pending\'`, as in the [default block parameter](#the-default-block-parameter).',
        example: fromDecimal(668)
      },
      {
        type: Quantity,
        desc: 'The uncle\'s index position.',
        example: fromDecimal(0)
      }
    ],
    returns: 'See [eth_getBlockByHash](#eth_getblockbyhash)'
  },

  getUncleCountByBlockHash: {
    desc: 'Returns the number of uncles in a block from a block matching the given block hash.',
    params: [
      {
        type: Hash,
        desc: '32 Bytes - hash of a block.',
        example: '0xb903239f8543d04b5dc1ba6579132b143087c68db1b2168786408fcbce568238'
      }
    ],
    returns: {
      type: Quantity,
      desc: 'integer of the number of uncles in this block.',
      example: fromDecimal(0)
    }
  },

  getUncleCountByBlockNumber: {
    desc: 'Returns the number of uncles in a block from a block matching the given block number.',
    params: [
      {
        type: BlockNumber,
        desc: 'integer of a block number, or the string \'latest\', \'earliest\' or \'pending\', see the [default block parameter](#the-default-block-parameter).',
        example: fromDecimal(232)
      }
    ],
    returns: {
      type: Quantity,
      desc: 'integer of the number of uncles in this block.',
      example: fromDecimal(1)
    }
  },

  getWork: {
    desc: 'Returns the hash of the current block, the seedHash, and the boundary condition to be met ("target").',
    params: [],
    returns: {
      type: Array,
      desc: 'Array with the following properties:\n  - `Data`, 32 Bytes - current block header pow-hash.\n  - `Data`, 32 Bytes - the seed hash used for the DAG.\n  - `Data`, 32 Bytes - the boundary condition ("target"), 2^256 / difficulty.\n  - `Quantity`, the current block number.',
      example: [
        '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
        '0x5EED00000000000000000000000000005EED0000000000000000000000000000',
        '0xd1ff1c01710000000000000000000000d1ff1c01710000000000000000000000',
        fromDecimal(1)
      ]
    }
  },

  hashrate: {
    desc: 'Returns the number of hashes per second that the node is mining with.',
    params: [],
    returns: {
      type: Quantity,
      desc: 'number of hashes per second.',
      example: fromDecimal(906)
    }
  },

  inspectTransaction: {
    nodoc: 'Not present in Rust code',
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  mining: {
    desc: 'Returns `true` if client is actively mining new blocks.',
    params: [],
    returns: {
      type: Boolean,
      desc: '`true` of the client is mining, otherwise `false`.',
      example: true
    }
  },

  newBlockFilter: {
    desc: 'Creates a filter in the node, to notify when a new block arrives.\nTo check if the state has changed, call [eth_getFilterChanges](#eth_getfilterchanges).',
    params: [],
    returns: {
      type: Quantity,
      desc: 'A filter id.',
      example: fromDecimal(1)
    }
  },

  newFilter: {
    desc: `Creates a filter object, based on filter options, to notify when the state changes (logs).
           To check if the state has changed, call [eth_getFilterChanges](#eth_getfilterchanges).

           ##### A note on specifying topic filters:
           Topics are order-dependent. A transaction with a log with topics [A, B] will be matched by the following topic filters:
           * \`[]\` "anything"
           * \`[A]\` "A in first position (and anything after)"
           * \`[null, B]\` "anything in first position AND B in second position (and anything after)"
           * \`[A, B]\` "A in first position AND B in second position (and anything after)"
           * \`[[A, B], [A, B]]\` "(A OR B) in first position AND (A OR B) in second position (and anything after)"`.replace(/^[^\S\n]+/gm, ''),
    params: [{
      type: Object,
      desc: 'The filter options:',
      details: {
        fromBlock: {
          type: BlockNumber,
          desc: 'Integer block number, or `\'latest\'` for the last mined block or `\'pending\'`, `\'earliest\'` for not yet mined transactions.',
          optional: true,
          default: 'latest'
        },
        toBlock: {
          type: BlockNumber,
          desc: 'Integer block number, or `\'latest\'` for the last mined block or `\'pending\'`, `\'earliest\'` for not yet mined transactions.',
          optional: true,
          default: 'latest'
        },
        address: {
          type: Address,
          desc: '20 Bytes - Contract address or a list of addresses from which logs should originate.',
          optional: true
        },
        topics: {
          type: Array,
          desc: 'Array of 32 Bytes `Data` topics. Topics are order-dependent. It\'s possible to pass in `null` to match any topic, or a subarray of multiple topics of which one should be matching.',
          optional: true
        },
        limit: {
          type: Quantity,
          desc: 'The maximum number of entries to retrieve (latest first).',
          optional: true
        }
      },
      example: {
        fromBlock: fromDecimal(1),
        toBlock: fromDecimal(2),
        address: '0x8888f1f195afa192cfee860698584c030f4c9db1',
        topics: withComment([
          withComment('0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b', 'This topic in first position'),
          withComment(null, 'Any topic in second position'),
          withComment(['0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b', '0x000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc'], 'Either topic of the two in third position')
        ], '... and anything after')
      }
    }],
    returns: {
      type: Quantity,
      desc: 'The filter id.',
      example: fromDecimal(1)
    }
  },

  newFilterEx: {
    nodoc: 'Not present in Rust code', // https://github.com/ethereum/wiki/wiki/Proposal:-Reversion-Notification
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  newPendingTransactionFilter: {
    desc: 'Creates a filter in the node, to notify when new pending transactions arrive.\n\nTo check if the state has changed, call [eth_getFilterChanges](#eth_getfilterchanges).',
    params: [],
    returns: {
      type: Quantity,
      desc: 'A filter id.',
      example: fromDecimal(1)
    }
  },

  notePassword: {
    nodoc: 'Not present in Rust code',
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  pendingTransactions: {
    nodoc: 'Not present in Rust code',
    desc: '?',
    params: [],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  protocolVersion: {
    desc: 'Returns the current ethereum protocol version.',
    params: [],
    returns: {
      type: String,
      desc: 'The current ethereum protocol version.',
      example: fromDecimal(99)
    }
  },

  register: {
    nodoc: 'Not present in Rust code',
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  sendRawTransaction: {
    desc: 'Creates new message call transaction or a contract creation for signed transactions.\n\n**Note:** `eth_submitTransaction` is an alias of this method.',
    params: [
      {
        type: Data,
        desc: 'The signed transaction data.',
        example: '0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675'
      }
    ],
    returns: {
      type: Hash,
      desc: '32 Bytes - the transaction hash, or the zero hash if the transaction is not yet available\n\nUse [eth_getTransactionReceipt](#eth_gettransactionreceipt) to get the contract address, after the transaction was mined, when you created a contract.',
      example: '0xe670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331'
    }
  },

  sendTransaction: {
    desc: 'Creates new message call transaction or a contract creation, if the data field contains code.',
    params: [
      {
        type: TransactionRequest,
        desc: 'The transaction object.',
        format: 'inputTransactionFormatter',
        example: {
          from: '0xb60e8dd61c5d32be8058bb8eb970870f07233155',
          to: '0xd46e8dd67c5d32be8058bb8eb970870f07244567',
          gas: fromDecimal(30400),
          gasPrice: fromDecimal(10000000000000),
          value: fromDecimal(2441406250),
          data: '0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675'
        }
      }
    ],
    returns: {
      type: Hash,
      desc: '32 Bytes - the transaction hash, or the zero hash if the transaction is not yet available.\n\nUse [eth_getTransactionReceipt](#eth_gettransactionreceipt) to get the contract address, after the transaction was mined, when you created a contract.',
      example: '0xe670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331'
    }
  },

  sign: {
    desc: 'The sign method calculates an Ethereum specific signature with: `sign(keccak256("\x19Ethereum Signed Message:\n" + len(message) + message)))`.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - address.',
        format: 'inputAddressFormatter',
        example: '0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826'
      },
      {
        type: Data,
        desc: 'Data which hash to sign.',
        example: withComment('0x5363686f6f6c627573', 'Schoolbus')
      }
    ],
    returns: {
      type: Data,
      desc: 'Signed data.',
      example: '0xb1092cb5b23c2aa55e5b5787729c6be812509376de99a52bea2b41e5a5f8601c5641e74d01e4493c17bf1ef8b179c49362b2c721222128d58422a539310c6ecd1b'
    }
  },

  signTransaction: {
    desc: 'Signs transactions without dispatching it to the network. It can be later submitted using [eth_sendRawTransaction](#eth_sendrawtransaction).',
    params: [
      {
        type: TransactionRequest,
        desc: 'Transaction object, see [eth_sendTransaction](#eth_sendTransaction).',
        format: 'inputCallFormatter',
        example: new Dummy('{ ... }')
      }
    ],
    returns: {
      type: Object,
      desc: 'Signed transaction and it\'s details:',
      details: {
        raw: {
          type: Data,
          desc: 'The signed, RLP encoded transaction.'
        },
        tx: {
          type: Object,
          desc: 'Transaction object:',
          details: {
            hash: {
              type: Hash,
              desc: '32 Bytes - hash of the transaction.'
            },
            nonce: {
              type: Quantity,
              desc: 'the number of transactions made by the sender prior to this one.'
            },
            blockHash: {
              type: Hash,
              desc: '32 Bytes - hash of the block where this transaction was in. `null` when its pending.'
            },
            blockNumber: {
              type: BlockNumber,
              desc: 'block number where this transaction was in. `null` when its pending.'
            },
            transactionIndex: {
              type: Quantity,
              desc: 'integer of the transactions index position in the block. `null` when its pending.'
            },
            from: {
              type: Address,
              desc: '20 Bytes - address of the sender.'
            },
            to: {
              type: Address,
              desc: '20 Bytes - address of the receiver. `null` when its a contract creation transaction.'
            },
            value: {
              type: Quantity,
              desc: 'value transferred in Wei.'
            },
            gasPrice: {
              type: Quantity,
              desc: 'gas price provided by the sender in Wei.'
            },
            gas: {
              type: Quantity,
              desc: 'gas provided by the sender.'
            },
            input: {
              type: Data,
              desc: 'the data send along with the transaction.'
            },
            v: {
              type: Quantity,
              desc: 'the standardised V field of the signature.'
            },
            standard_v: {
              type: Quantity,
              desc: 'the standardised V field of the signature (0 or 1).'
            },
            r: {
              type: Quantity,
              desc: 'the R field of the signature.'
            },
            raw: {
              type: Data,
              desc: 'raw transaction data'
            },
            publicKey: {
              type: Hash,
              desc: 'public key of the signer.'
            },
            chainId: {
              type: Quantity,
              desc: 'the chain id of the transaction, if any.'
            },
            creates: {
              type: Hash,
              desc: 'creates contract hash'
            },
            condition: {
              type: Object,
              optional: true,
              desc: 'conditional submission, Block number in `block` or timestamp in `time` or `null`. (parity-feature)'
            }
          }
        }
      },
      example: {
        raw: '0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675',
        tx: {
          hash: '0xc6ef2fc5426d6ad6fd9e2a26abeab0aa2411b7ab17f30a99d3cb96aed1d1055b',
          nonce: fromDecimal(0),
          blockHash: '0xbeab0aa2411b7ab17f30a99d3cb9c6ef2fc5426d6ad6fd9e2a26a6aed1d1055b',
          blockNumber: fromDecimal(5599),
          transactionIndex: fromDecimal(1),
          from: '0x407d73d8a49eeb85d32cf465507dd71d507100c1',
          to: '0x853f43d8a49eeb85d32cf465507dd71d507100c1',
          value: fromDecimal(520464),
          gas: fromDecimal(520464),
          gasPrice: '0x09184e72a000',
          input: '0x603880600c6000396000f300603880600c6000396000f3603880600c6000396000f360'
        }
      }
    }
  },

  submitWork: {
    desc: 'Used for submitting a proof-of-work solution.',
    params: [
      {
        type: Data,
        desc: '8 Bytes - The nonce found (64 bits).',
        example: '0x0000000000000001'
      },
      {
        type: Data,
        desc: '32 Bytes - The header\'s pow-hash (256 bits)',
        example: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
      },
      {
        type: Data,
        desc: '32 Bytes - The mix digest (256 bits).',
        example: '0xD1FE5700000000000000000000000000D1FE5700000000000000000000000000'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the provided solution is valid, otherwise `false`.',
      example: true
    }
  },

  submitHashrate: {
    desc: 'Used for submitting mining hashrate.',
    params: [
      {
        type: Data,
        desc: 'a hexadecimal string representation (32 bytes) of the hash rate.',
        example: '0x0000000000000000000000000000000000000000000000000000000000500000'
      },
      {
        type: Data,
        desc: 'A random hexadecimal(32 bytes) ID identifying the client.',
        example: '0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c'
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if submitting went through succesfully and `false` otherwise.',
      example: true
    }
  },

  syncing: {
    desc: 'Returns an object with data about the sync status or `false`.',
    params: [],
    returns: {
      type: Object,
      desc: 'An object with sync status data or `FALSE`, when not syncing.',
      format: 'outputSyncingFormatter',
      details: {
        startingBlock: {
          type: Quantity,
          desc: 'The block at which the import started (will only be reset, after the sync reached this head)'
        },
        currentBlock: {
          type: Quantity,
          desc: 'The current block, same as eth_blockNumber'
        },
        highestBlock: {
          type: Quantity,
          desc: 'The estimated highest block'
        },
        blockGap: {
          type: Array,
          desc: 'Array of "first", "last", such that [first, last) are all missing from the chain'
        },
        warpChunksAmount: {
          type: Quantity,
          desc: 'Total amount of snapshot chunks'
        },
        warpChunksProcessed: {
          type: Quantity,
          desc: 'Total amount of snapshot chunks processed'
        }
      },
      example: withComment({
        startingBlock: fromDecimal(900),
        currentBlock: fromDecimal(902),
        highestBlock: fromDecimal(1108)
      }, 'Or `false` when not syncing')
    }
  },

  uninstallFilter: {
    desc: 'Uninstalls a filter with given id. Should always be called when watch is no longer needed.\nAdditonally Filters timeout when they aren\'t requested with [eth_getFilterChanges](#eth_getfilterchanges) for a period of time.',
    params: [
      {
        type: Quantity,
        desc: 'The filter id.',
        example: fromDecimal(11)
      }
    ],
    returns: {
      type: Boolean,
      desc: '`true` if the filter was successfully uninstalled, otherwise `false`.',
      example: true
    }
  },

  unregister: {
    nodoc: 'Not present in Rust code',
    desc: '?',
    params: [
      '?'
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful'
    }
  },

  // Pub-Sub
  subscribe: {
    subdoc: SUBDOC_PUBSUB,
    desc: `
Starts a subscription (on WebSockets / IPC / TCP transports) to a particular event. For every event that
matches the subscription a JSON-RPC notification with event details and subscription ID will be sent to a client.

An example notification received by subscribing to \`newHeads\` event:
\`\`\`
{"jsonrpc":"2.0","method":"eth_subscription","params":{"subscription":"0x416d77337e24399d","result":{"difficulty":"0xd9263f42a87",<...>,
"uncles":[]}}}
\`\`\`

You can unsubscribe using \`eth_unsubscribe\` RPC method. Subscriptions are also tied to a transport
connection, disconnecting causes all subscriptions to be canceled.
    `,
    params: [
      {
        type: String,
        desc: 'Subscription type: one of `newHeads`, `logs`',
        example: 'newHeads'
      },
      {
        type: Object,
        desc: `
Subscription type-specific parameters. It must be left empty for
\`newHeads\` and must contain filter object for \`logs\`.
        `,
        example: {
          fromBlock: 'latest',
          toBlock: 'latest'
        }
      }
    ],
    returns: {
      type: String,
      desc: 'Assigned subscription ID',
      example: '0x416d77337e24399d'
    }
  },
  unsubscribe: {
    subdoc: SUBDOC_PUBSUB,
    desc: 'Unsubscribes from a subscription.',
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
});
