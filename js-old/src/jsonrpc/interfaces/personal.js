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

import { Address, Data, Quantity, TransactionRequest } from '../types';

export default {
  listAccounts: {
    desc: 'Lists all stored accounts.',
    params: [],
    returns: {
      type: Array,
      desc: 'A list of 20 byte account identifiers.',
      example: [
        '0x7bf87721a96849d168de02fd6ea5986a3a147383',
        '0xca807a90fd64deed760fb98bf0869b475c469348'
      ]
    }
  },

  newAccount: {
    desc: 'Creates new account.\n\n**Note:** it becomes the new current unlocked account. There can only be one unlocked account at a time.',
    params: [
      {
        type: String,
        desc: 'Password for the new account.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Address,
      desc: '20 Bytes - The identifier of the new account.',
      example: '0x8f0227d45853a50eefd48dd4fec25d5b3fd2295e'
    }
  },

  sendTransaction: {
    desc: 'Sends transaction and signs it in a single call. The account does not need to be unlocked to make this call, and will not be left unlocked after.',
    params: [
      {
        type: TransactionRequest,
        desc: 'The transaction object',
        example: {
          from: '0x407d73d8a49eeb85d32cf465507dd71d507100c1',
          to: '0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b',
          data: '0x41cd5add4fd13aedd64521e363ea279923575ff39718065d38bd46f0e6632e8e',
          value: '0x186a0'
        }
      },
      {
        type: String,
        desc: 'Passphrase to unlock the `from` account.',
        example: 'hunter2'
      }
    ],
    returns: {
      type: Data,
      desc: '32 Bytes - the transaction hash, or the zero hash if the transaction is not yet available',
      example: '0x62e05075829655752e146a129a044ad72e95ce33e48ff48118b697e15e7b41e4'
    }
  },

  unlockAccount: {
    desc: 'Unlocks specified account for use.\n\nIf permanent unlocking is disabled (the default) then the duration argument will be ignored, and the account will be unlocked for a single signing. With permanent locking enabled, the duration sets the number of seconds to hold the account open for. It will default to 300 seconds. Passing 0 unlocks the account indefinitely.\n\nThere can only be one unlocked account at a time.',
    params: [
      {
        type: Address,
        desc: '20 Bytes - The address of the account to unlock.',
        example: '0x8f0227d45853a50eefd48dd4fec25d5b3fd2295e'
      },
      {
        type: String,
        desc: 'Passphrase to unlock the account.',
        example: 'hunter2'
      },
      {
        type: Quantity,
        default: 300,
        desc: 'Integer or `null` - Duration in seconds how long the account should remain unlocked for.',
        example: null
      }
    ],
    returns: {
      type: Boolean,
      desc: 'whether the call was successful',
      example: true
    }
  }
};
