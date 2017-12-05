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

import { inAddress, inBlockNumber, inData, inFilter, inHash, inHex, inNumber16, inOptions } from '../../format/input';
import { outAddress, outBlock, outLog, outNumber, outReceipt, outSyncing, outTransaction } from '../../format/output';

export default class Eth {
  constructor (transport) {
    this._transport = transport;
  }

  accounts () {
    return this._transport
      .execute('eth_accounts')
      .then((accounts) => (accounts || []).map(outAddress));
  }

  blockNumber () {
    return this._transport
      .execute('eth_blockNumber')
      .then(outNumber);
  }

  call (options, blockNumber = 'latest') {
    return this._transport
      .execute('eth_call', inOptions(options), inBlockNumber(blockNumber));
  }

  coinbase () {
    return this._transport
      .execute('eth_coinbase')
      .then(outAddress);
  }

  compileLLL (code) {
    return this._transport
      .execute('eth_compileLLL', inData(code));
  }

  compileSerpent (code) {
    return this._transport
      .execute('eth_compileSerpent', inData(code));
  }

  compileSolidity (code) {
    return this._transport
      .execute('eth_compileSolidity', inData(code));
  }

  estimateGas (options) {
    return this._transport
      .execute('eth_estimateGas', inOptions(options))
      .then(outNumber);
  }

  fetchQueuedTransactions () {
    return this._transport
      .execute('eth_fetchQueuedTransactions');
  }

  flush () {
    return this._transport
      .execute('eth_flush');
  }

  gasPrice () {
    return this._transport
      .execute('eth_gasPrice')
      .then(outNumber);
  }

  getBalance (address, blockNumber = 'latest') {
    return this._transport
      .execute('eth_getBalance', inAddress(address), inBlockNumber(blockNumber))
      .then(outNumber);
  }

  getBlockByHash (hash, full = false) {
    return this._transport
      .execute('eth_getBlockByHash', inHex(hash), full)
      .then(outBlock);
  }

  getBlockByNumber (blockNumber = 'latest', full = false) {
    return this._transport
      .execute('eth_getBlockByNumber', inBlockNumber(blockNumber), full)
      .then(outBlock);
  }

  getBlockTransactionCountByHash (hash) {
    return this._transport
      .execute('eth_getBlockTransactionCountByHash', inHex(hash))
      .then(outNumber);
  }

  getBlockTransactionCountByNumber (blockNumber = 'latest') {
    return this._transport
      .execute('eth_getBlockTransactionCountByNumber', inBlockNumber(blockNumber))
      .then(outNumber);
  }

  getCode (address, blockNumber = 'latest') {
    return this._transport
      .execute('eth_getCode', inAddress(address), inBlockNumber(blockNumber));
  }

  getCompilers () {
    return this._transport
      .execute('eth_getCompilers');
  }

  getFilterChanges (filterId) {
    return this._transport
      .execute('eth_getFilterChanges', inNumber16(filterId))
      .then((logs) => logs.map(outLog));
  }

  getFilterChangesEx (filterId) {
    return this._transport
      .execute('eth_getFilterChangesEx', inNumber16(filterId));
  }

  getFilterLogs (filterId) {
    return this._transport
      .execute('eth_getFilterLogs', inNumber16(filterId))
      .then((logs) => logs.map(outLog));
  }

  getFilterLogsEx (filterId) {
    return this._transport
      .execute('eth_getFilterLogsEx', inNumber16(filterId));
  }

  getLogs (options) {
    return this._transport
      .execute('eth_getLogs', inFilter(options))
      .then((logs) => logs.map(outLog));
  }

  getLogsEx (options) {
    return this._transport
      .execute('eth_getLogsEx', inFilter(options));
  }

  getStorageAt (address, index = 0, blockNumber = 'latest') {
    return this._transport
      .execute('eth_getStorageAt', inAddress(address), inNumber16(index), inBlockNumber(blockNumber));
  }

  getTransactionByBlockHashAndIndex (hash, index = 0) {
    return this._transport
      .execute('eth_getTransactionByBlockHashAndIndex', inHex(hash), inNumber16(index))
      .then(outTransaction);
  }

  getTransactionByBlockNumberAndIndex (blockNumber = 'latest', index = 0) {
    return this._transport
      .execute('eth_getTransactionByBlockNumberAndIndex', inBlockNumber(blockNumber), inNumber16(index))
      .then(outTransaction);
  }

  getTransactionByHash (hash) {
    return this._transport
      .execute('eth_getTransactionByHash', inHex(hash))
      .then(outTransaction);
  }

  getTransactionCount (address, blockNumber = 'latest') {
    return this._transport
      .execute('eth_getTransactionCount', inAddress(address), inBlockNumber(blockNumber))
      .then(outNumber);
  }

  getTransactionReceipt (txhash) {
    return this._transport
      .execute('eth_getTransactionReceipt', inHex(txhash))
      .then(outReceipt);
  }

  getUncleByBlockHashAndIndex (hash, index = 0) {
    return this._transport
      .execute('eth_getUncleByBlockHashAndIndex', inHex(hash), inNumber16(index));
  }

  getUncleByBlockNumberAndIndex (blockNumber = 'latest', index = 0) {
    return this._transport
      .execute('eth_getUncleByBlockNumberAndIndex', inBlockNumber(blockNumber), inNumber16(index));
  }

  getUncleCountByBlockHash (hash) {
    return this._transport
      .execute('eth_getUncleCountByBlockHash', inHex(hash))
      .then(outNumber);
  }

  getUncleCountByBlockNumber (blockNumber = 'latest') {
    return this._transport
      .execute('eth_getUncleCountByBlockNumber', inBlockNumber(blockNumber))
      .then(outNumber);
  }

  getWork () {
    return this._transport
      .execute('eth_getWork');
  }

  hashrate () {
    return this._transport
      .execute('eth_hashrate')
      .then(outNumber);
  }

  inspectTransaction () {
    return this._transport
      .execute('eth_inspectTransaction');
  }

  mining () {
    return this._transport
      .execute('eth_mining');
  }

  newBlockFilter () {
    return this._transport
      .execute('eth_newBlockFilter');
  }

  newFilter (options) {
    return this._transport
      .execute('eth_newFilter', inFilter(options));
  }

  newFilterEx (options) {
    return this._transport
      .execute('eth_newFilterEx', inFilter(options));
  }

  newPendingTransactionFilter () {
    return this._transport
      .execute('eth_newPendingTransactionFilter');
  }

  notePassword () {
    return this._transport
      .execute('eth_notePassword');
  }

  pendingTransactions () {
    return this._transport
      .execute('eth_pendingTransactions');
  }

  protocolVersion () {
    return this._transport
      .execute('eth_protocolVersion');
  }

  register () {
    return this._transport
      .execute('eth_register');
  }

  sendRawTransaction (data) {
    return this._transport
      .execute('eth_sendRawTransaction', inData(data));
  }

  sendTransaction (options) {
    return this._transport
      .execute('eth_sendTransaction', inOptions(options));
  }

  sign (address, hash) {
    return this._transport
      .execute('eth_sign', inAddress(address), inHash(hash));
  }

  signTransaction (options) {
    return this._transport
      .execute('eth_signTransaction', inOptions(options));
  }

  submitHashrate (hashrate, clientId) {
    return this._transport
      .execute('eth_submitHashrate', inNumber16(hashrate), clientId);
  }

  submitWork (nonce, powHash, mixDigest) {
    return this._transport
      .execute('eth_submitWork', inNumber16(nonce), powHash, mixDigest);
  }

  syncing () {
    return this._transport
      .execute('eth_syncing')
      .then(outSyncing);
  }

  uninstallFilter (filterId) {
    return this._transport
      .execute('eth_uninstallFilter', inHex(filterId));
  }

  unregister () {
    return this._transport
      .execute('eth_unregister');
  }
}
