import BigNumber from 'bignumber.js';

import { isInstanceOf, isString } from '../util/types';

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
  return inHex(data);
}

export function inFilter (options) {
  if (options) {
    Object.keys(options).forEach((key) => {
      switch (key) {
        case 'address':
          options[key] = inAddress(options[key]);
          break;

        case 'fromBlock':
        case 'toBlock':
          options[key] = inBlockNumber(options[key]);
          break;
      }
    });
  }

  return options;
}

export function inHex (str) {
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
