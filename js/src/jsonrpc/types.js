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

export class Address {}

export class Data {}

export class Hash {}

export class Integer {}

export class Float {}

export class Quantity {}

export class BlockNumber {
  static print = '`Quantity` | `Tag`';
}

export class CallRequest {
  static print = '`Object`';

  static details = {
    from: {
      type: Address,
      desc: '20 Bytes - The address the transaction is send from.',
      optional: true
    },
    to: {
      type: Address,
      desc: '(optional when creating new contract) 20 Bytes - The address the transaction is directed to.'
    },
    gas: {
      type: Quantity,
      desc: 'Integer of the gas provided for the transaction execution. eth_call consumes zero gas, but this parameter may be needed by some executions.',
      optional: true
    },
    gasPrice: {
      type: Quantity,
      desc: 'Integer of the gas price used for each paid gas.',
      optional: true
    },
    value: {
      type: Quantity,
      desc: 'Integer of the value sent with this transaction.',
      optional: true
    },
    data: {
      type: Data,
      desc: '4 byte hash of the method signature followed by encoded parameters. For details see [Ethereum Contract ABI](https://github.com/ethereum/wiki/wiki/Ethereum-Contract-ABI).',
      optional: true
    }
  }
}

export class TransactionRequest {
  static print = '`Object`';

  static details = {
    from: {
      type: Address,
      desc: '20 Bytes - The address the transaction is send from.'
    },
    to: {
      type: Address,
      desc: '20 Bytes - The address the transaction is directed to.',
      optional: true
    },
    gas: {
      type: Quantity,
      desc: 'Integer of the gas provided for the transaction execution. eth_call consumes zero gas, but this parameter may be needed by some executions.',
      optional: true
    },
    gasPrice: {
      type: Quantity,
      desc: 'Integer of the gas price used for each paid gas.',
      optional: true
    },
    value: {
      type: Quantity,
      desc: 'Integer of the value sent with this transaction.',
      optional: true
    },
    data: {
      type: Data,
      desc: '4 byte hash of the method signature followed by encoded parameters. For details see [Ethereum Contract ABI](https://github.com/ethereum/wiki/wiki/Ethereum-Contract-ABI).',
      optional: true
    },
    nonce: {
      type: Quantity,
      desc: 'Integer of a nonce. This allows to overwrite your own pending transactions that use the same nonce.',
      optional: true
    },
    condition: {
      type: Object,
      desc: 'Conditional submission of the transaction. Can be either an integer block number `{ block: 1 }` or UTC timestamp (in seconds) `{ time: 1491290692 }` or `null`.',
      optional: true
    }
  }
}

export class TransactionResponse {
  static print = '`Object`';

  static details = {
    hash: {
      type: Hash,
      desc: '32 Bytes - hash of the transaction.'
    },
    nonce: {
      type: Quantity,
      desc: 'The number of transactions made by the sender prior to this one.'
    },
    blockHash: {
      type: Hash,
      desc: '32 Bytes - hash of the block where this transaction was in. `null` when its pending.'
    },
    blockNumber: {
      type: BlockNumber,
      desc: 'Block number where this transaction was in. `null` when its pending.'
    },
    transactionIndex: {
      type: Quantity,
      desc: 'Integer of the transactions index position in the block. `null` when its pending.'
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
      desc: 'Value transferred in Wei.'
    },
    gasPrice: {
      type: Quantity,
      desc: 'Gas price provided by the sender in Wei.'
    },
    gas: {
      type: Quantity,
      desc: 'Gas provided by the sender.'
    },
    input: {
      type: Data,
      desc: 'The data send along with the transaction.'
    },
    creates: {
      type: Address,
      optional: true,
      desc: 'Address of a created contract or `null`.'
    },
    raw: {
      type: Data,
      desc: 'Raw transaction data.'
    },
    publicKey: {
      type: Data,
      desc: 'Public key of the signer.'
    },
    chainId: {
      type: Quantity,
      desc: 'The chain id of the transaction, if any.'
    },
    standardV: {
      type: Quantity,
      desc: 'The standardized V field of the signature (0 or 1).'
    },
    v: {
      type: Quantity,
      desc: 'The V field of the signature.'
    },
    r: {
      type: Quantity,
      desc: 'The R field of the signature.'
    },
    s: {
      type: Quantity,
      desc: 'The S field of the signature.'
    },
    condition: {
      type: Object,
      optional: true,
      desc: 'Conditional submission, Block number in `block` or timestamp in `time` or `null`.'
    }
  }
}
