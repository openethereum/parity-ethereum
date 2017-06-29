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

import { inAddress, inBlockNumber, inData, inHex, inNumber16, inOptions, inFilter, inDeriveHash, inDeriveIndex } from '../format/input';
import { outAccountInfo, outAddress, outBlock, outChainStatus, outHistogram, outHwAccountInfo, outNodeKind, outNumber, outPeers, outTransaction, outSyncing, outReceipt, outLog, outAddresses, outRecentDapps, outVaultMeta } from '../format/output';

import PubsubBase from './pubsubBase';

export default class Pubsub extends PubsubBase {
  constructor (transport) {
    super(transport);
    this._api = 'parity';
  }

  unsubscribe (subscriptionIds) {
    return this.removeListener(subscriptionIds);
  }

  // eth API
  // `newHeads`, `logs`, `newPendingTransactions`, `syncing`
  newHeads (callback) {
    return this.addListener('eth', 'newHeads', callback);
  }
  //
  // logs (callback) {
  //   throw Error('not supported yet');
  // }
  //
  // newPendingTransactions (callback) {
  //   throw Error('not supported yet');
  // }
  //
  // syncing (callback) {
  //   throw Error('not supported yet');
  // }

  // parity API
  accountsInfo (callback) {
    return this.addListener(this._api, 'parity_accountsInfo', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAccountInfo(data));
    });
  }

  hardwareAccountsInfo (callback) {
    return this.addListener(this._api, 'parity_hardwareAccountsInfo', (error, data) => {
      error
        ? callback(error)
        : callback(null, outHwAccountInfo(data));
    });
  }

  defaultAccount (callback) {
    return this.addListener(this._api, 'parity_defaultAccount', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    });
  }

  transactionsLimit (callback) {
    return this.addListener(this._api, 'parity_transactionsLimit', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  extraData (callback) {
    return this.addListener(this._api, 'parity_extraData', callback);
  }

  gasFloorTarget (callback) {
    return this.addListener(this._api, 'parity_gasFloorTarget', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  gasCeilTarget (callback) {
    return this.addListener(this._api, 'parity_gasCeilTarget', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  minGasPrice (callback) {
    return this.addListener(this._api, 'parity_minGasPrice', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  devLogs (callback) {
    return this.addListener(this._api, 'parity_devLogs', callback);
  }

  devLogsLevels (callback) {
    return this.addListener(this._api, 'parity_devLogsLevels', callback);
  }

  netChain (callback) {
    return this.addListener(this._api, 'parity_netChain', callback);
  }

  netPeers (callback) {
    return this.addListener(this._api, 'parity_netPeers', (error, data) => {
      error
        ? callback(error)
        : callback(null, outPeers(data));
    });
  }

  netPort (callback) {
    return this.addListener(this._api, 'parity_netPort', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  rpcSettings (callback) {
    return this.addListener(this._api, 'parity_rpcSettings', callback);
  }

  nodeName (callback) {
    return this.addListener(this._api, 'parity_nodeName', callback);
  }

  defaultExtraData (callback) {
    return this.addListener(this._api, 'parity_defaultExtraData', callback);
  }

  gasPriceHistogram (callback) {
    return this.addListener(this._api, 'parity_gasPriceHistogram', (error, data) => {
      error
        ? callback(error)
        : callback(null, outHistogram(data));
    });
  }

  unsignedTransactionsCount (callback) {
    return this.addListener(this._api, 'parity_unsignedTransactionsCount', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  registryAddress (callback) {
    return this.addListener(this._api, 'parity_registryAddress', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    });
  }

  listAccounts (callback, count, offset = null, blockNumber = 'latest') {
    return this.addListener(this._api, 'parity_listAccounts', (error, data) => {
      error
        ? callback(error)
        : callback(null, (data) => (data || []).map(outAddress));
    }, [count, inAddress(offset), inBlockNumber(blockNumber)]);
  }

  listStorageKeys (callback, address, count, hash = null, blockNumber = 'latest') {
    return this.addListener(this._api, 'parity_listStorageKeys', callback, [inAddress(address), count, inHex(hash), inBlockNumber(blockNumber)]);
  }

  pendingTransactions (callback) {
    return this.addListener(this._api, 'parity_pendingTransactions', (error, data) => {
      error
        ? callback(error)
        : callback(null, outTransaction(data));
    });
  }

  futureTransactions (callback) {
    return this.addListener(this._api, 'parity_futureTransactions', (error, data) => {
      error
        ? callback(error)
        : callback(null, outTransaction(data));
    });
  }

  pendingTransactionsStats (callback) {
    return this.addListener(this._api, 'parity_pendingTransactionsStats', callback);
  }

  localTransactions (callback) {
    return this.addListener(this._api, 'parity_localTransactions', (error, transactions) => {
      error
        ? callback(error)
        : callback(null, transactions => {
          Object.values(transactions)
            .filter(tx => tx.transaction)
            .map(tx => {
              tx.transaction = outTransaction(tx.transaction);
            });
          return transactions;
        });
    });
  }

  dappsUrl (callback) {
    return this.addListener(this._api, 'parity_dappsUrl', callback);
  }

  wsUrl (callback) {
    return this.addListener(this._api, 'parity_wsUrl', callback);
  }

  nextNonce (callback, account) {
    return this.addListener(this._api, 'parity_nextNonce', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    }, [inAddress(account)]);
  }

  mode (callback) {
    return this.addListener(this._api, 'parity_mode', callback);
  }

  chain (callback) {
    return this.addListener(this._api, 'parity_chain', callback);
  }

  enode (callback) {
    return this.addListener(this._api, 'parity_enode', callback);
  }

  consensusCapability (callback) {
    return this.addListener(this._api, 'parity_consensusCapability', callback);
  }

  versionInfo (callback) {
    return this.addListener(this._api, 'parity_versionInfo', callback);
  }

  releasesInfo (callback) {
    return this.addListener(this._api, 'parity_releasesInfo', callback);
  }

  chainStatus (callback) {
    return this.addListener(this._api, 'parity_chainStatus', (error, data) => {
      error
        ? callback(error)
        : callback(null, outChainStatus(data));
    });
  }

  nodeKind (callback) {
    return this.addListener(this._api, 'parity_nodeKind', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNodeKind(data));
    });
  }

  getBlockHeaderByNumber (callback, blockNumber = 'latest') {
    return this.addListener(this._api, 'parity_getBlockHeaderByNumber', (error, data) => {
      error
        ? callback(error)
        : callback(null, outBlock(data));
    }, [inBlockNumber(blockNumber)]);
  }

  cidV0 (callback, data) {
    return this.addListener(this._api, 'parity_cidV0', callback, [inData(data)]);
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

  // net API
  version (callback) {
    return this.addListener(this._api, 'net_version', callback);
  }

  peerCount (callback) {
    return this.addListener(this._api, 'net_peerCount', (error, data) => {
      error
        ? callback(error)
        : callback(null, outNumber(data));
    });
  }

  listening (callback) {
    return this.addListener(this._api, 'net_listening', callback);
  }

  // parity accounts API (only secure API or configured to be exposed)
  allAccountsInfo (callback) {
    return this._addListener(this._api, 'parity_allAccountsInfo', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAccountInfo(data));
    });
  }

  getDappAddresses (callback, dappId) {
    return this._addListener(this._api, 'parity_getDappAddresses', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddresses(data));
    }, [dappId]);
  }

  getDappDefaultAddress (callback, dappId) {
    return this._addListener(this._api, 'parity_getDappDefaultAddress', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    }, [dappId]);
  }

  getNewDappsAddresses (callback) {
    return this._addListener(this._api, 'parity_getDappDefaultAddress', (error, addresses) => {
      error
        ? callback(error)
        : callback(null, addresses ? addresses.map(outAddress) : null);
    });
  }

  getNewDappsDefaultAddress (callback) {
    return this._addListener(this._api, 'parity_getNewDappsDefaultAddress', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    });
  }

  listRecentDapps (callback) {
    return this._addListener(this._api, 'parity_listRecentDapps', (error, data) => {
      error
        ? callback(error)
        : callback(null, outRecentDapps(data));
    });
  }

  listGethAccounts (callback) {
    return this._addListener(this._api, 'parity_listGethAccounts', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddresses(data));
    });
  }

  listVaults (callback) {
    return this._addListener(this._api, 'parity_listVaults', callback);
  }

  listOpenedVaults (callback) {
    return this._addListener(this._api, 'parity_listOpenedVaults', callback);
  }

  getVaultMeta (callback, vaultName) {
    return this._addListener(this._api, 'parity_getVaultMeta', (error, data) => {
      error
        ? callback(error)
        : callback(null, outVaultMeta(data));
    }, [vaultName]);
  }

  deriveAddressHash (callback, address, password, hash, shouldSave) {
    return this._addListener(this._api, 'parity_deriveAddressHash', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    }, [inAddress(address), password, inDeriveHash(hash), !!shouldSave]);
  }

  deriveAddressIndex (callback, address, password, index, shouldSave) {
    return this._addListener(this._api, 'parity_deriveAddressIndex', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    }, [inAddress(address), password, inDeriveIndex(index), !!shouldSave]);
  }
}
