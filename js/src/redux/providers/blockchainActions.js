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

import Contracts from '~/contracts';

export function setBlock (blockNumber, block) {
  return {
    type: 'setBlock',
    blockNumber, block
  };
}

export function setTransaction (txHash, info) {
  return {
    type: 'setTransaction',
    txHash, info
  };
}

export function setBytecode (address, bytecode) {
  return {
    type: 'setBytecode',
    address, bytecode
  };
}

export function setMethod (signature, method) {
  return {
    type: 'setMethod',
    signature, method
  };
}

export function fetchBlock (blockNumber) {
  return (dispatch, getState) => {
    const { blocks } = getState().blockchain;

    if (blocks[blockNumber.toString()]) {
      return;
    }

    const { api } = getState();

    api.eth
      .getBlockByNumber(blockNumber)
      .then(block => {
        dispatch(setBlock(blockNumber, block));
      })
      .catch(e => {
        console.error('blockchain::fetchBlock', e);
      });
  };
}

export function fetchTransaction (txHash) {
  return (dispatch, getState) => {
    const { transactions } = getState().blockchain;

    if (transactions[txHash]) {
      return;
    }

    const { api } = getState();

    api.eth
      .getTransactionByHash(txHash)
      .then(info => {
        dispatch(setTransaction(txHash, info));
      })
      .catch(e => {
        console.error('blockchain::fetchTransaction', e);
      });
  };
}

export function fetchBytecode (address) {
  return (dispatch, getState) => {
    const { bytecodes } = getState().blockchain;

    if (bytecodes[address]) {
      return;
    }

    const { api } = getState();

    api.eth
      .getCode(address)
      .then(code => {
        dispatch(setBytecode(address, code));
      })
      .catch(e => {
        console.error('blockchain::fetchBytecode', e);
      });
  };
}

export function fetchMethod (signature) {
  return (dispatch, getState) => {
    const { methods } = getState().blockchain;

    if (methods[signature]) {
      return;
    }

    Contracts
      .get()
      .signatureReg.lookup(signature)
      .then(method => {
        dispatch(setMethod(signature, method));
      })
      .catch(e => {
        console.error('blockchain::fetchMethod', e);
      });
  };
}
