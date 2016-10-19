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

import BigNumber from 'bignumber.js';

import { toChecksumAddress } from '../../abi/util/address';

export function outAccountInfo (infos) {
  const ret = {};

  Object.keys(infos).forEach((address) => {
    const info = infos[address];

    ret[outAddress(address)] = {
      name: info.name,
      uuid: info.uuid,
      meta: JSON.parse(info.meta)
    };
  });

  return ret;
}

export function outAddress (address) {
  return toChecksumAddress(address);
}

export function outBlock (block) {
  if (block) {
    Object.keys(block).forEach((key) => {
      switch (key) {
        case 'author':
        case 'miner':
          block[key] = outAddress(block[key]);
          break;

        case 'difficulty':
        case 'gasLimit':
        case 'gasUsed':
        case 'nonce':
        case 'number':
        case 'totalDifficulty':
          block[key] = outNumber(block[key]);
          break;

        case 'timestamp':
          block[key] = outDate(block[key]);
          break;
      }
    });
  }

  return block;
}

export function outDate (date) {
  return new Date(outNumber(date).toNumber() * 1000);
}

export function outLog (log) {
  Object.keys(log).forEach((key) => {
    switch (key) {
      case 'blockNumber':
      case 'logIndex':
      case 'transactionIndex':
        log[key] = outNumber(log[key]);
        break;

      case 'address':
        log[key] = outAddress(log[key]);
        break;
    }
  });

  return log;
}

export function outNumber (number) {
  return new BigNumber(number || 0);
}

export function outPeers (peers) {
  return {
    active: outNumber(peers.active),
    connected: outNumber(peers.connected),
    max: outNumber(peers.max)
  };
}

export function outReceipt (receipt) {
  if (receipt) {
    Object.keys(receipt).forEach((key) => {
      switch (key) {
        case 'blockNumber':
        case 'cumulativeGasUsed':
        case 'gasUsed':
        case 'transactionIndex':
          receipt[key] = outNumber(receipt[key]);
          break;

        case 'contractAddress':
          receipt[key] = outAddress(receipt[key]);
          break;
      }
    });
  }

  return receipt;
}

export function outSignerRequest (request) {
  if (request) {
    Object.keys(request).forEach((key) => {
      switch (key) {
        case 'id':
          request[key] = outNumber(request[key]);
          break;

        case 'payload':
          request[key].transaction = outTransaction(request[key].transaction);
          break;
      }
    });
  }

  return request;
}

export function outTransaction (tx) {
  if (tx) {
    Object.keys(tx).forEach((key) => {
      switch (key) {
        case 'blockNumber':
        case 'gasPrice':
        case 'gas':
        case 'nonce':
        case 'transactionIndex':
        case 'value':
          tx[key] = outNumber(tx[key]);
          break;

        case 'creates':
        case 'from':
        case 'to':
          tx[key] = outAddress(tx[key]);
          break;
      }
    });
  }

  return tx;
}
