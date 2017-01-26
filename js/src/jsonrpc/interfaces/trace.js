// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { BlockNumber, Data, Hash, Integer } from '../types';
import { withPreamble, Dummy } from '../helpers';

const SECTION_FILTERING = 'Transaction-Trace Filtering';
const SECTION_ADHOC = 'Ad-hoc Tracing';

export default withPreamble(`

The trace module is for getting a deeper insight into transaction processing.
It includes two sets of calls; the transaction trace filtering API and the ad-hoc tracing API.

## The Ad-hoc Tracing API

The ad-hoc tracing API allows you to perform a number of different diagnostics on calls or transactions,
eitherhistorical ones from the chain or hypothetical ones not yet mined. The diagnostics include:

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

In order to use these API Parity must be fully synced with flags \`$ parity --tracing on\`.

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
    desc: 'Returns traces created at given block',
    params: [
      {
        type: BlockNumber,
        desc: 'Integer block number, or \'latest\' for the last mined block or \'pending\', \'earliest\' for not yet mined transactions'
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces'
    }
  },

  filter: {
    section: SECTION_FILTERING,
    desc: 'Returns traces matching given filter',
    params: [
      {
        type: Object,
        desc: 'The filter object'
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces matching given filter'
    }
  },

  get: {
    section: SECTION_FILTERING,
    desc: 'Returns trace at given position.',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash'
      },
      {
        type: Integer,
        desc: 'Trace position witing transaction'
      }
    ],
    returns: {
      type: Object,
      desc: 'Trace object'
    }
  },

  transaction: {
    section: SECTION_FILTERING,
    desc: 'Returns all traces of given transaction',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash'
      }
    ],
    returns: {
      type: Array,
      desc: 'Traces of given transaction',
      example: new Dummy('[ ... ]')
    }
  },

  call: {
    section: SECTION_ADHOC,
    desc: 'Returns traces for a specific call',
    params: [
      {
        type: Object,
        desc: 'Call options'
      },
      {
        type: BlockNumber,
        desc: 'The blockNumber'
      },
      {
        type: Array,
        desc: 'Type of trace, one or more of \'vmTrace\', \'trace\' and/or \'stateDiff\''
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces'
    }
  },

  rawTransaction: {
    section: SECTION_ADHOC,
    desc: 'Traces a call to eth_sendRawTransaction without making the call, returning the traces',
    params: [
      {
        type: Data,
        desc: 'Transaction data'
      },
      {
        type: Array,
        desc: 'Type of trace, one or more of \'vmTrace\', \'trace\' and/or \'stateDiff\''
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces'
    }
  },

  replayTransaction: {
    section: SECTION_ADHOC,
    desc: 'Replays a transaction, returning the traces',
    params: [
      {
        type: Hash,
        desc: 'Transaction hash'
      },
      {
        type: Array,
        desc: 'Type of trace, one or more of \'vmTrace\', \'trace\' and/or \'stateDiff\''
      }
    ],
    returns: {
      type: Array,
      desc: 'Block traces'
    }
  }
});
