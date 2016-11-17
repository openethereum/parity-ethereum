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

import { isArray, isHex, isInstanceOf, isString } from '../util/types';

export function inAddress (address) {
  // TODO: address validation if we have upper-lower addresses
  return inHex(address);
}

export function inBlockNumber (blockNumber) {
  if (isString(blockNumber)) {
    switch (blockNumber) {
      case 'earliest':
      case 'latest':
      case 'pending':
        return blockNumber;
    }
  }

  return inNumber16(blockNumber);
}

export function inData (data) {
  if (data && data.length && !isHex(data)) {
    data = data.split('').map((chr) => {
      return `0${chr.charCodeAt(0).toString(16)}`.slice(-2);
    }).join('');
  }

  return inHex(data);
}

export function inHash (hash) {
  return inHex(hash);
}

export function inTopics (_topics) {
  let topics = (_topics || [])
    .filter((topic) => topic)
    .map(inHex);

  while (topics.length < 4) {
    topics.push(null);
  }

  return topics;
}

export function inFilter (options) {
  if (options) {
    Object.keys(options).forEach((key) => {
      switch (key) {
        case 'address':
          if (isArray(options[key])) {
            options[key] = options[key].map(inAddress);
          } else {
            options[key] = inAddress(options[key]);
          }
          break;

        case 'fromBlock':
        case 'toBlock':
          options[key] = inBlockNumber(options[key]);
          break;

        case 'limit':
          options[key] = inNumber10(options[key]);
          break;

        case 'topics':
          options[key] = inTopics(options[key]);
      }
    });
  }

  return options;
}

export function inHex (str) {
  if (str && str.toString) {
    str = str.toString(16);
  }

  if (str && str.substr(0, 2) === '0x') {
    return str.toLowerCase();
  }

  return `0x${(str || '').toLowerCase()}`;
}

export function inNumber10 (number) {
  if (isInstanceOf(number, BigNumber)) {
    return number.toNumber();
  }

  return (new BigNumber(number || 0)).toNumber();
}

export function inNumber16 (number) {
  if (isInstanceOf(number, BigNumber)) {
    return inHex(number.toString(16));
  }

  return inHex((new BigNumber(number || 0)).toString(16));
}

export function inOptions (options) {
  if (options) {
    Object.keys(options).forEach((key) => {
      switch (key) {
        case 'from':
        case 'to':
          options[key] = inAddress(options[key]);
          break;

        case 'gas':
        case 'gasPrice':
        case 'value':
        case 'nonce':
          options[key] = inNumber16(options[key]);
          break;

        case 'data':
          options[key] = inData(options[key]);
          break;
      }
    });
  }

  return options;
}

export function inTraceFilter (filterObject) {
  if (filterObject) {
    Object.keys(filterObject).forEach((key) => {
      switch (key) {
        case 'fromAddress':
        case 'toAddress':
          filterObject[key] = [].concat(filterObject[key])
            .map(address => inAddress(address));
          break;

        case 'toBlock':
        case 'fromBlock':
          filterObject[key] = inBlockNumber(filterObject[key]);
          break;
      }
    });
  }

  return filterObject;
}

export function inTraceType (whatTrace) {
  if (isString(whatTrace)) {
    return [whatTrace];
  }

  return whatTrace;
}
