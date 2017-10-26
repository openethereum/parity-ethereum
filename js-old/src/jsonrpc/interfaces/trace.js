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

import { Address, BlockNumber, Data, Hash, CallRequest } from '../types';
import { withPreamble, Dummy, fromDecimal } from '../helpers';

const SECTION_FILTERING = 'Transaction-Trace Filtering';
const SECTION_ADHOC = 'Ad-hoc Tracing';

export default withPreamble(`

The trace module is for getting a deeper insight into transaction processing.
It includes two sets of calls; the transaction trace filtering API and the ad-hoc tracing API.

**Note:** In order to use these API Parity must be fully synced with flags \`$ parity --tracing on\`.

## The Ad-hoc Tracing API

The ad-hoc tracing API allows you to perform a number of different diagnostics on calls or transactions,
either historical ones from the chain or hypothetical ones not yet mined. The diagnostics include:

- \`trace\` **Transaction trace**. An equivalent trace to that in the previous section.
- \`vmTrace\` **Virtual Machine execution trace**. Provides a full trace of the VM's state throughout the execution of the transaction, including for any subcalls.
- \`stateDiff\` **State difference**. Provides information detailing all altered portions of the Ethereum state made due to the execution of the transaction.

There are three means of providing a transaction to execute; either providing the same information as when making
a call using \`eth_call\` (see \`trace_call\`), through providing raw, signed, transaction data as when using
\`eth_sendRawTransaction\` (see \`trace_rawTransaction\`) or simply a transaction hash for a previously mined
transaction (see \`trace_replayTransaction\`). In the latter case, your node must be in archive mode or the
transaction should be within the most recent 1000 blocks.

## The Transaction-Trace Filtering API

These APIs allow you to get a full *externality* trace on any transaction executed throughout the Parity chain.
Unlike the log filtering API, you are able to search and filter based only upon address information.
Information returned includes the execution of all \`CREATE\`s, \`SUICIDE\`s and all variants of \`CALL\` together
with input data, output data, gas usage, amount transferred and the success status of each individual action.

### \`traceAddress\` field

The \`traceAddress\` field of all returned traces, gives the exact location in the call trace [index in root,
index in first \`CALL\`, index in second \`CALL\`, ...].

i.e. if the trace is:
\`\`\`
A
  CALLs B
    CALLs G
  CALLs C
    CALLs G
\`\`\`
then it should look something like:

\`[ {A: []}, {B: [0]}, {G: [0, 0]}, {C: [1]}, {G: [1, 0]} ]\`

`, {
  block: {
    section: SECTION_FILTERING,
    desc: 'Returns traces created at given block.',
    params: [
      {
        type: BlockNumber,
        desc: 'Integer of a block number, or the string `\'earliest\'`, `\'latest\'` or `\'pending\'`.',
        example: fromDecimal(3068185)
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces.',
      example: [
        {
          action: {
            callType: 'call',
            from: '0xaa7b131dc60b80d3cf5e59b5a21a666aa039c951',
            gas: '0x0',
            input: '0x',
            to: '0xd40aba8166a212d6892125f079c33e6f5ca19814',
            value: '0x4768d7effc3fbe'
          },
          blockHash: '0x7eb25504e4c202cf3d62fd585d3e238f592c780cca82dacb2ed3cb5b38883add',
          blockNumber: 3068185,
          result: {
            gasUsed: '0x0',
            output: '0x'
          },
          subtraces: 0,
          traceAddress: [],
          transactionHash: '0x07da28d752aba3b9dd7060005e554719c6205c8a3aea358599fc9b245c52f1f6',
          transactionPosition: 0,
          type: 'call'
        },
        new Dummy('...')
      ]
    }
  },

  filter: {
    section: SECTION_FILTERING,
    desc: 'Returns traces matching given filter',
    params: [
      {
        type: Object,
        desc: 'The filter object',
        details: {
          fromBlock: {
            type: BlockNumber,
            desc: 'From this block.',
            optional: true
          },
          toBlock: {
            type: BlockNumber,
            desc: 'To this block.',
            optional: true
          },
          fromAddress: {
            type: Array,
            desc: 'Sent from these addresses.',
            optional: true
          },
          toAddress: {
            type: Address,
            desc: 'Sent to these addresses.',
            optional: true
          }
        },
        example: {
          fromBlock: fromDecimal(3068100),
          toBlock: fromDecimal(3068200),
          toAddress: ['0x8bbB73BCB5d553B5A556358d27625323Fd781D37']
        }
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces matching given filter',
      example: [
        {
          action: {
            callType: 'call',
            from: '0x32be343b94f860124dc4fee278fdcbd38c102d88',
            gas: '0x4c40d',
            input: '0x',
            to: '0x8bbb73bcb5d553b5a556358d27625323fd781d37',
            value: '0x3f0650ec47fd240000'
          },
          blockHash: '0x86df301bcdd8248d982dbf039f09faf792684e1aeee99d5b58b77d620008b80f',
          blockNumber: 3068183,
          result: {
            gasUsed: '0x0',
            output: '0x'
          },
          subtraces: 0,
          traceAddress: [],
          transactionHash: '0x3321a7708b1083130bd78da0d62ead9f6683033231617c9d268e2c7e3fa6c104',
          transactionPosition: 3,
          type: 'call'
        },
        new Dummy('...')
      ]
    }
  },

  get: {
    section: SECTION_FILTERING,
    desc: 'Returns trace at given position.',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash.',
        example: '0x17104ac9d3312d8c136b7f44d4b8b47852618065ebfa534bd2d3b5ef218ca1f3'
      },
      {
        type: Array,
        desc: 'Index positions of the traces.',
        example: ['0x0']
      }
    ],
    returns: {
      type: Object,
      desc: 'Trace object',
      example: {
        action: {
          callType: 'call',
          from: '0x1c39ba39e4735cb65978d4db400ddd70a72dc750',
          gas: '0x13e99',
          input: '0x16c72721',
          to: '0x2bd2326c993dfaef84f696526064ff22eba5b362',
          value: '0x0'
        },
        blockHash: '0x7eb25504e4c202cf3d62fd585d3e238f592c780cca82dacb2ed3cb5b38883add',
        blockNumber: 3068185,
        result: {
          gasUsed: '0x183',
          output: '0x0000000000000000000000000000000000000000000000000000000000000001'
        },
        subtraces: 0,
        traceAddress: [0],
        transactionHash: '0x17104ac9d3312d8c136b7f44d4b8b47852618065ebfa534bd2d3b5ef218ca1f3',
        transactionPosition: 2,
        type: 'call'
      }
    }
  },

  transaction: {
    section: SECTION_FILTERING,
    desc: 'Returns all traces of given transaction',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash',
        example: '0x17104ac9d3312d8c136b7f44d4b8b47852618065ebfa534bd2d3b5ef218ca1f3'
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces of given transaction',
      example: [
        {
          action: {
            callType: 'call',
            from: '0x1c39ba39e4735cb65978d4db400ddd70a72dc750',
            gas: '0x13e99',
            input: '0x16c72721',
            to: '0x2bd2326c993dfaef84f696526064ff22eba5b362',
            value: '0x0'
          },
          blockHash: '0x7eb25504e4c202cf3d62fd585d3e238f592c780cca82dacb2ed3cb5b38883add',
          blockNumber: 3068185,
          result: {
            gasUsed: '0x183',
            output: '0x0000000000000000000000000000000000000000000000000000000000000001'
          },
          subtraces: 0,
          traceAddress: [0],
          transactionHash: '0x17104ac9d3312d8c136b7f44d4b8b47852618065ebfa534bd2d3b5ef218ca1f3',
          transactionPosition: 2,
          type: 'call'
        },
        new Dummy('...')
      ]
    }
  },

  call: {
    section: SECTION_ADHOC,
    desc: 'Executes the given call and returns a number of possible traces for it.',
    params: [
      {
        type: CallRequest,
        desc: 'Call options, same as `eth_call`.',
        example: new Dummy('{ ... }')
      },
      {
        type: Array,
        desc: 'Type of trace, one or more of: `"vmTrace"`, `"trace"`, `"stateDiff"`.',
        example: ['trace']
      },
      {
        type: BlockNumber,
        optional: true,
        desc: 'Integer of a block number, or the string `\'earliest\'`, `\'latest\'` or `\'pending\'`.'
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces',
      example: {
        output: '0x',
        stateDiff: null,
        trace: [
          {
            action: new Dummy('{ ... }'),
            result: {
              gasUsed: '0x0',
              output: '0x'
            },
            subtraces: 0,
            traceAddress: [],
            type: 'call'
          }
        ],
        vmTrace: null
      }
    }
  },

  rawTransaction: {
    section: SECTION_ADHOC,
    desc: 'Traces a call to `eth_sendRawTransaction` without making the call, returning the traces',
    params: [
      {
        type: Data,
        desc: 'Raw transaction data.',
        example: '0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675'
      },
      {
        type: Array,
        desc: 'Type of trace, one or more of: `"vmTrace"`, `"trace"`, `"stateDiff"`.',
        example: ['trace']
      }
    ],
    returns: {
      type: Object,
      desc: 'Block traces.',
      example: {
        output: '0x',
        stateDiff: null,
        trace: [
          {
            action: new Dummy('{ ... }'),
            result: {
              gasUsed: '0x0',
              output: '0x'
            },
            subtraces: 0,
            traceAddress: [],
            type: 'call'
          }
        ],
        vmTrace: null
      }
    }
  },

  replayTransaction: {
    section: SECTION_ADHOC,
    desc: 'Replays a transaction, returning the traces.',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash.',
        example: '0x02d4a872e096445e80d05276ee756cefef7f3b376bcec14246469c0cd97dad8f'
      },
      {
        type: Array,
        desc: 'Type of trace, one or more of: `"vmTrace"`, `"trace"`, `"stateDiff"`.',
        example: ['trace']
      }
    ],
    returns: {
      type: Object,
      desc: 'Block traces.',
      example: {
        output: '0x',
        stateDiff: null,
        trace: [
          {
            action: new Dummy('{ ... }'),
            result: {
              gasUsed: '0x0',
              output: '0x'
            },
            subtraces: 0,
            traceAddress: [],
            type: 'call'
          }
        ],
        vmTrace: null
      }
    }
  }
});
