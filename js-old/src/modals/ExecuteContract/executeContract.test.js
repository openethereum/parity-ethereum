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

import BigNumber from 'bignumber.js';
import sinon from 'sinon';

const CONTRACT = {
  functions: [
    {
      name: 'test_a',
      signature: 'test_a',
      estimateGas: sinon.stub().resolves(new BigNumber(123)),
      inputs: [
        {
          name: 'test_bool',
          kind: {
            type: 'bool'
          }
        }
      ],
      abi: {
        inputs: [
          {
            name: 'test_bool',
            type: 'bool'
          }
        ]
      }
    }
  ]
};

const STORE = {
  dispatch: sinon.stub(),
  subscribe: sinon.stub(),
  getState: () => {
    return {
      balances: {
        balances: {}
      },
      nodeStatus: {
        gasLimit: new BigNumber(123)
      },
      personal: {
        accountsInfo: {}
      },
      settings: {
        backgroundSeed: ''
      },
      registry: {
        reverse: {}
      }
    };
  }
};

function createApi (result = true) {
  const sha3 = sinon.stub().resolves('0x0000000000000000000000000000000000000000');

  sha3.text = sha3;
  return {
    parity: {
      registryAddress: sinon.stub().resolves('0x0000000000000000000000000000000000000000')
    },
    util: { sha3 }
  };
}

export {
  createApi,
  CONTRACT,
  STORE
};
