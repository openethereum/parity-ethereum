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

import { inAddress, inBlockNumber, inData, inDeriveHash, inDeriveIndex, inHex, inNumber16, inOptions, inFilter } from '../format/input';
import { outAccountInfo, outAddress, outAddresses, outBlock, outChainStatus, outHistogram, outHwAccountInfo, outNodeKind, outNumber, outPeers, outRecentDapps, outTransaction, outVaultMeta, outSyncing, outReceipt, outLog } from '../format/output';

import Provider from './provider';

export default class ParityProvider extends Provider {
  constructor (transport) {
    super(transport);
    this._api = 'parity_subscribe';
  }

  unsubscribe (...subscriptionIds) {
    return this._removeListener('parity_unsubscribe', subscriptionIds);
  }

  // parity API
  accountsInfo (callback) {
    return this._addListener(this._api, 'parity_accountsInfo', callback)
                .then(outAccountInfo);
  }

  hardwareAccountsInfo (callback) {
    return this._addListener(this._api, 'parity_hardwareAccountsInfo', callback)
                .then(outHwAccountInfo);
  }

  defaultAccount (callback) {
    return this._addListener(this._api, 'parity_defaultAccount', callback)
                .then(outAddress);
  }

  transactionsLimit (callback) {
    return this._addListener(this._api, 'parity_transactionsLimit', callback)
                .then(outNumber);
  }

  extraData (callback) {
    return this._addListener(this._api, 'parity_extraData', callback);
  }

  gasFloorTarget (callback) {
    return this._addListener(this._api, 'parity_gasFloorTarget', callback)
                .then(outNumber);
  }

  gasCeilTarget (callback) {
    return this._addListener(this._api, 'parity_gasCeilTarget', callback)
                .then(outNumber);
  }

  minGasPrice (callback) {
    return this._addListener(this._api, 'parity_minGasPrice', callback)
                .then(outNumber);
  }

  devLogs (callback) {
    return this._addListener(this._api, 'parity_devLogs', callback);
  }

  devLogsLevels (callback) {
    return this._addListener(this._api, 'parity_devLogsLevels', callback);
  }

  netChain (callback) {
    return this._addListener(this._api, 'parity_netChain', callback);
  }

  netPeers (callback) {
    return this._addListener(this._api, 'parity_netPeers', callback)
                .then(outPeers);
  }

  netPort (callback) {
    return this._addListener(this._api, 'parity_netPort', callback)
                .then(outNumber);
  }

  rpcSettings (callback) {
    return this._addListener(this._api, 'parity_rpcSettings', callback);
  }

  nodeName (callback) {
    return this._addListener(this._api, 'parity_nodeName', callback);
  }

  defaultExtraData (callback) {
    return this._addListener(this._api, 'parity_defaultExtraData', callback);
  }

  gasPriceHistogram (callback) {
    return this._addListener(this._api, 'parity_gasPriceHistogram', callback)
                .then(outHistogram);
  }

  unsignedTransactionsCount (callback) {
    return this._addListener(this._api, 'parity_unsignedTransactionsCount', callback)
                .then(outNumber);
  }

  generateSecretPhrase (callback) {
    return this._addListener(this._api, 'parity_generateSecretPhrase', callback);
  }

  phraseToAddress (callback, phrase) {
    return this._addListener(this._api, 'parity_phraseToAddress', callback, phrase)
                .then(outAddress);
  }

  registryAddress (callback) {
    return this._addListener(this._api, 'parity_registryAddress', callback)
                .then(outAddress);
  }

  listAccounts (callback, count, offset = null, blockNumber = 'latest') {
    return this._addListener(this._api, 'parity_listAccounts', callback, count, inAddress(offset), inBlockNumber(blockNumber))
               .then((accounts) => (accounts || []).map(outAddress));
  }

  listStorageKeys (callback, address, count, hash = null, blockNumber = 'latest') {
    return this._addListener(this._api, 'parity_listStorageKeys', callback, inAddress(address), count, inHex(hash), inBlockNumber(blockNumber));
  }

  encryptMessage (callback, pubkey, data) {
    return this._addListener(this._api, 'parity_encryptMessage', callback, inHex(pubkey), inHex(data));
  }

  pendingTransactions (callback) {
    return this._addListener(this._api, 'parity_pendingTransactions', callback)
                .then(data => data.map(outTransaction));
  }

  futureTransactions (callback) {
    return this._addListener(this._api, 'parity_futureTransactions', callback)
                .then(data => data.map(outTransaction));
  }

  pendingTransactionsStats (callback) {
    return this._addListener(this._api, 'parity_pendingTransactionsStats', callback);
  }

  localTransactions (callback) {
    return this._addListener(this._api, 'parity_localTransactions', callback)
                .then(transactions => {
                  Object.values(transactions)
                    .filter(tx => tx.transaction)
                    .map(tx => {
                      tx.transaction = outTransaction(tx.transaction);
                    });
                  return transactions;
                });
  }

  dappsUrl (callback) {
    return this._addListener(this._api, 'parity_dappsUrl', callback);
  }

  wsUrl (callback) {
    return this._addListener(this._api, 'parity_wsUrl', callback);
  }

  nextNonce (callback, account) {
    return this._addListener(this._api, 'parity_nextNonce', callback, inAddress(account))
                .then(outNumber);
  }

  mode (callback) {
    return this._addListener(this._api, 'parity_mode', callback);
  }

  chain (callback) {
    return this._addListener(this._api, 'parity_chain', callback);
  }

  enode (callback) {
    return this._addListener(this._api, 'parity_enode', callback);
  }

  consensusCapability (callback) {
    return this._addListener(this._api, 'parity_consensusCapability', callback);
  }

  versionInfo (callback) {
    return this._addListener(this._api, 'parity_versionInfo', callback);
  }

  releasesInfo (callback) {
    return this._addListener(this._api, 'parity_releasesInfo', callback);
  }

  chainStatus (callback) {
    return this._addListener(this._api, 'parity_chainStatus', callback)
                .then(outChainStatus);
  }

  nodeKind (callback) {
    return this._addListener(this._api, 'parity_nodeKind', callback)
                .then(outNodeKind);
  }

  getBlockHeaderByNumber (callback, blockNumber = 'latest') {
    return this._addListener(this._api, 'parity_getBlockHeaderByNumber', callback, inBlockNumber(blockNumber))
                .then(outBlock);
  }

  cidV0 (callback, data) {
    return this._addListener(this._api, 'parity_cidV0', callback, inData(data));
  }

  //  eth API

  protocolVersion (callback) {
    return this._addListener(this._api, 'eth_protocolVersion', callback);
  }

  syncing (callback) {
    return this._addListener(this._api, 'eth_syncing', callback)
                .then(outSyncing);
  }

  hashrate (callback) {
    return this._addListener(this._api, 'eth_hashrate', callback)
                .then(outNumber);
  }

  coinbase (callback) {
    return this._addListener(this._api, 'eth_coinbase', callback)
                .then(outAddress);
  }

  mining (callback) {
    return this._addListener(this._api, 'eth_mining', callback);
  }

  gasPrice (callback) {
    return this._addListener(this._api, 'eth_gasPrice', callback)
                .then(outNumber);
  }

  accounts (callback) {
    return this._addListener(this._api, 'eth_accounts', callback)
                .then((accounts) => (accounts || []).map(outAddress));
  }

  blockNumber (callback) {
    return this._addListener(this._api, 'eth_blockNumber', callback)
                .then(outNumber);
  }

  getBalance (callback, address, blockNumber = 'latest') {
    return this._addListener(this._api, 'eth_getBalance', callback, inAddress(address), inBlockNumber(blockNumber))
    .then(outNumber);
  }

  getStorageAt (callback, address, index = 0, blockNumber = 'latest') {
    return this._addListener(this._api, 'eth_getStorageAt', callback, inAddress(address), inNumber16(index), inBlockNumber(blockNumber));
  }

  getBlockByHash (callback, hash, full = false) {
    return this._addListener(this._api, 'eth_getBlockByHash', callback, inHex(hash), full)
                .then(outBlock);
  }

  getBlockByNumber (callback, blockNumber = 'latest', full = false) {
    return this._addListener(this._api, 'eth_getBlockByNumber', callback, inBlockNumber(blockNumber), full)
                .then(outBlock);
  }

  getTransactionCount (callback, address, blockNumber = 'latest') {
    return this._addListener(this._api, 'eth_getTransactionCount', callback, inAddress(address), inBlockNumber(blockNumber))
                .then(outNumber);
  }

  getBlockTransactionCountByHash (callback, hash) {
    return this._addListener(this._api, 'eth_getBlockTransactionCountByHash', callback, inHex(hash))
                .then(outNumber);
  }

  getBlockTransactionCountByNumber (callback, blockNumber = 'latest') {
    return this._addListener(this._api, 'eth_getBlockTransactionCountByNumber', callback, inBlockNumber(blockNumber))
                .then(outNumber);
  }

  getUncleCountByBlockHash (callback, hash) {
    return this._addListener(this._api, 'eth_getUncleCountByBlockHash', callback, inHex(hash))
                .then(outNumber);
  }

  getUncleCountByBlockNumber (callback, blockNumber = 'latest') {
    return this._addListener(this._api, 'eth_getUncleCountByBlockNumber', callback, inBlockNumber(blockNumber))
                .then(outNumber);
  }

  getCode (callback, address, blockNumber = 'latest') {
    return this._addListener(this._api, 'eth_getCode', callback, inAddress(address), inBlockNumber(blockNumber));
  }

  sendRawTransaction (callback, data) {
    return this._addListener(this._api, 'eth_sendRawTransaction', callback, inData(data));
  }

  submitTransaction (callback, data) {
    return this._addListener(this._api, 'eth_submitTransaction', callback, inData(data));
  }

  call (callback, options, blockNumber = 'latest') {
    return this._addListener(this._api, 'eth_call', callback, inOptions(options), inBlockNumber(blockNumber));
  }

  estimateGas (callback, options) {
    return this._addListener(this._api, 'eth_estimateGas', callback, inOptions(options))
                .then(outNumber);
  }

  getTransactionByHash (callback, hash) {
    return this._addListener(this._api, 'eth_getTransactionByHash', callback, inHex(hash))
    .then(outTransaction);
  }

  getTransactionByBlockHashAndIndex (callback, hash, index = 0) {
    return this._addListener(this._api, 'eth_getTransactionByBlockHashAndIndex', callback, inHex(hash), inNumber16(index))
    .then(outTransaction);
  }

  getTransactionByBlockNumberAndIndex (callback, blockNumber = 'latest', index = 0) {
    return this._addListener(this._api, 'eth_getTransactionByBlockNumberAndIndex', callback, inBlockNumber(blockNumber), inNumber16(index))
    .then(outTransaction);
  }

  getTransactionReceipt (callback, txhash) {
    return this._addListener(this._api, 'eth_getTransactionReceipt', callback, inHex(txhash))
    .then(outReceipt);
  }

  getUncleByBlockHashAndIndex (callback, hash, index = 0) {
    return this._addListener(this._api, 'eth_getUncleByBlockHashAndIndex', callback, inHex(hash), inNumber16(index));
  }

  getUncleByBlockNumberAndIndex (callback, blockNumber = 'latest', index = 0) {
    return this._addListener(this._api, 'eth_getUncleByBlockNumberAndIndex', callback, inBlockNumber(blockNumber), inNumber16(index));
  }

  getLogs (callback, options) {
    return this._addListener(this._api, 'eth_getLogs', callback, inFilter(options))
    .then((logs) => logs.map(outLog));
  }

  getWork (callback) {
    return this._addListener(this._api, 'eth_getWork', callback);
  }

  submitWork (callback, nonce, powHash, mixDigest) {
    return this._addListener(this._api, 'eth_submitWork', callback, inNumber16(nonce), powHash, mixDigest);
  }

  submitHashrate (callback, hashrate, clientId) {
    return this._addListener(this._api, 'eth_submitHashrate', callback, inNumber16(hashrate), clientId);
  }

  // net API

  version (callback) {
    return this._addListener(this._api, 'net_version', callback);
  }

  peerCount (callback) {
    return this._addListener(this._api, 'net_peerCount', callback)
                .then(outNumber);
  }

  listening (callback) {
    return this._addListener(this._api, 'net_listening', callback);
  }

  // parity accounts API

  allAccountsInfo (callback) {
    return this._addListener(this._api, 'parity_allAccountsInfo', callback)
                .then(outAccountInfo);
  }

  getDappAddresses (callback, dappId) {
    return this._addListener(this._api, 'parity_getDappAddresses', callback, dappId)
                .then(outAddresses);
  }

  getDappDefaultAddress (callback, dappId) {
    return this._addListener(this._api, 'parity_getDappDefaultAddress', callback, dappId)
                .then(outAddresses);
  }

  getNewDappsAddresses (callback) {
    return this._addListener(this._api, 'parity_getDappDefaultAddress', callback)
    .then((addresses) => addresses ? addresses.map(outAddress) : null);
  }

  getNewDappsDefaultAddress (callback) {
    return this._addListener(this._api, 'parity_getNewDappsDefaultAddress', callback)
    .then(outAddress);
  }

  listRecentDapps (callback) {
    return this._addListener(this._api, 'parity_listRecentDapps', callback)
                .then(outRecentDapps);
  }

  listGethAccounts (callback) {
    return this._addListener(this._api, 'parity_listGethAccounts', callback)
    .then(outAddresses);
  }

  listVaults (callback) {
    return this._addListener(this._api, 'parity_listVaults', callback);
  }

  listOpenedVaults (callback) {
    return this._addListener(this._api, 'parity_listOpenedVaults', callback);
  }

  getVaultMeta (callback, vaultName) {
    return this._addListener(this._api, 'parity_getVaultMeta', callback, vaultName)
                .then(outVaultMeta);
  }

  deriveAddressHash (callback, address, password, hash, shouldSave) {
    return this._addListener(this._api, 'parity_deriveAddressHash', callback, inAddress(address), password, inDeriveHash(hash), !!shouldSave)
    .then(outAddress);
  }

  deriveAddressIndex (callback, address, password, index, shouldSave) {
    return this._addListener(this._api, 'parity_deriveAddressIndex', callback, inAddress(address), password, inDeriveIndex(index), !!shouldSave)
                .then(outAddress);
  }

  // Parity set API (not supported yet)
  // dappsList (callback) {
  //   return this._addListener(this._api, 'parity_dappsList', callback);
  // }
  //
  // hashContent (callbacK, url) {
  //   return this._addListener(this._api, 'parity_hashContent', callback, url);
  // }

  // personal API (not in default options)

  // listAccounts (callback) {
  //   return this._addListener(this._api, 'personal_listAccounts', callback, url);
  // }
}
