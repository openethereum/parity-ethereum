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
import PubsubBase from '../pubsubBase';

import { inAddress, inBlockNumber, inHex, inNumber16, inOptions, inFilter } from '../../format/input';
import { outAddress, outBlock, outNumber, outTransaction, outSyncing, outReceipt, outLog } from '../../format/output';

export default class Eth extends PubsubBase {
  constructor (transport) {
    super(transport);
    this._api = 'parity';
  }

  newHeads (callback) {
    return this.addListener('eth', 'newHeads', callback);
  }

  logs (callback) {
    throw Error('not supported yet');
  }

  //  eth API
  protocolVersion (callback) {
    return this.addListener(this._api, 'eth_protocolVersion', callback);
  }

  syncing (callback) {
    return this.addListener(this._api, 'eth_syncing', (error, data) => {
      error
        ? callback(error)
        : callback(null, outSyncing(data));
    });
  }

  hashrate (callback) {
    return this.addListener(this._api, 'eth_hashrate', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  coinbase (callback) {
    return this.addListener(this._api, 'eth_coinbase', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    });
  }

  mining (callback) {
    return this.addListener(this._api, 'eth_mining', callback);
  }

  gasPrice (callback) {
    return this.addListener(this._api, 'eth_gasPrice', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  accounts (callback) {
    return this.addListener(this._api, 'eth_accounts', (error, accounts) => {
      error
        ? callback(error)
        : callback(null, (accounts || []).map(outAddress));
    });
  }

  blockNumber (callback) {
    return this.addListener(this._api, 'eth_blockNumber', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  getBalance (callback, address, blockNumber = 'latest') {
    return this.addListener(this._api, 'eth_getBalance', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inAddress(address), inBlockNumber(blockNumber)]);
  }

  getStorageAt (callback, address, index = 0, blockNumber = 'latest') {
    return this.addListener(this._api, 'eth_getStorageAt', callback, [inAddress(address), inNumber16(index), inBlockNumber(blockNumber)]);
  }

  getBlockByHash (callback, hash, full = false) {
    return this.addListener(this._api, 'eth_getBlockByHash', (error, data) => {
      error
        ? callback(error)
        : callback(null, outBlock(data));
    }, [inHex(hash), full]);
  }

  getBlockByNumber (callback, blockNumber = 'latest', full = false) {
    return this.addListener(this._api, 'eth_getBlockByNumber', (error, data) => {
      error
        ? callback(error)
        : callback(null, outBlock(data));
    }, [inBlockNumber(blockNumber), full]);
  }

  getTransactionCount (callback, address, blockNumber = 'latest') {
    return this.addListener(this._api, 'eth_getTransactionCount', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inAddress(address), inBlockNumber(blockNumber)]);
  }

  getBlockTransactionCountByHash (callback, hash) {
    return this.addListener(this._api, 'eth_getBlockTransactionCountByHash', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inHex(hash)]);
  }

  getBlockTransactionCountByNumber (callback, blockNumber = 'latest') {
    return this.addListener(this._api, 'eth_getBlockTransactionCountByNumber', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inBlockNumber(blockNumber)]);
  }

  getUncleCountByBlockHash (callback, hash) {
    return this.addListener(this._api, 'eth_getUncleCountByBlockHash', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inHex(hash)]);
  }

  getUncleCountByBlockNumber (callback, blockNumber = 'latest') {
    return this.addListener(this._api, 'eth_getUncleCountByBlockNumber', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inBlockNumber(blockNumber)]);
  }

  getCode (callback, address, blockNumber = 'latest') {
    return this.addListener(this._api, 'eth_getCode', callback, [inAddress(address), inBlockNumber(blockNumber)]);
  }

  call (callback, options, blockNumber = 'latest') {
    return this.addListener(this._api, 'eth_call', callback, [inOptions(options), inBlockNumber(blockNumber)]);
  }

  estimateGas (callback, options) {
    return this.addListener(this._api, 'eth_estimateGas', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inOptions(options)]);
  }

  getTransactionByHash (callback, hash) {
    return this.addListener(this._api, 'eth_getTransactionByHash', (error, data) => {
      error
        ? callback(error)
        : callback(null, outTransaction(data));
    }, [inHex(hash)]);
  }

  getTransactionByBlockHashAndIndex (callback, hash, index = 0) {
    return this.addListener(this._api, 'eth_getTransactionByBlockHashAndIndex', (error, data) => {
      error
        ? callback(error)
        : callback(null, outTransaction(data));
    }, [inHex(hash), inNumber16(index)]);
  }

  getTransactionByBlockNumberAndIndex (callback, blockNumber = 'latest', index = 0) {
    return this.addListener(this._api, 'eth_getTransactionByBlockNumberAndIndex', (error, data) => {
      error
        ? callback(error)
        : callback(null, outTransaction(data));
    }, [inBlockNumber(blockNumber), inNumber16(index)]);
  }

  getTransactionReceipt (callback, txhash) {
    return this.addListener(this._api, 'eth_getTransactionReceipt', (error, data) => {
      error
        ? callback(error)
        : callback(null, outReceipt(data));
    }, [inHex(txhash)]);
  }

  getUncleByBlockHashAndIndex (callback, hash, index = 0) {
    return this.addListener(this._api, 'eth_getUncleByBlockHashAndIndex', callback, [inHex(hash), inNumber16(index)]);
  }

  getUncleByBlockNumberAndIndex (callback, blockNumber = 'latest', index = 0) {
    return this.addListener(this._api, 'eth_getUncleByBlockNumberAndIndex', callback, [inBlockNumber(blockNumber), inNumber16(index)]);
  }

  getLogs (callback, options) {
    return this.addListener(this._api, 'eth_getLogs', (error, logs) => {
      error
        ? callback(error)
        : callback(null, (logs) => logs.map(outLog));
    }, [inFilter(options)]);
  }

  getWork (callback) {
    return this.addListener(this._api, 'eth_getWork', callback);
  }
}
