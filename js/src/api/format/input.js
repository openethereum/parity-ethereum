import BigNumber from 'bignumber.js';

import { isInstanceOf, isString } from '../util/types';

// const ZERO_64 = '0000000000000000000000000000000000000000000000000000000000000000';

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

  // const hex = inHex(data).substr(2);
  // const missing = hex.length % 64;
  //
  // return `0x${hex}${ZERO_64.slice(-1 * missing)}`;
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
          options[key] = inAddress(options[key]);
          break;

        case 'fromBlock':
        case 'toBlock':
          options[key] = inBlockNumber(options[key]);
          break;

        case 'topics':
          options[key] = inTopics(options[key]);
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
