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
import { inAddress, inBlockNumber, inData, inHex, inDeriveHash, inDeriveIndex } from '../../format/input';
import { outAccountInfo, outAddress, outBlock, outChainStatus, outHistogram, outHwAccountInfo, outNodeKind, outNumber, outPeers, outTransaction, outAddresses, outRecentDapps, outVaultMeta } from '../../format/output';

export default class Parity extends PubsubBase {
  constructor (transport) {
    super(transport);
    this._api = 'parity';
  }

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
      if (error) {
        return callback(error);
      }

      Object.values(transactions)
        .filter(tx => tx.transaction)
        .map(tx => {
          tx.transaction = outTransaction(tx.transaction);
        });

      callback(null, transactions);
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

  // parity accounts API (only secure API or configured to be exposed)
  allAccountsInfo (callback) {
    return this.addListener(this._api, 'parity_allAccountsInfo', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAccountInfo(data));
    });
  }

  getDappAddresses (callback, dappId) {
    return this.addListener(this._api, 'parity_getDappAddresses', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddresses(data));
    }, [dappId]);
  }

  getDappDefaultAddress (callback, dappId) {
    return this.addListener(this._api, 'parity_getDappDefaultAddress', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    }, [dappId]);
  }

  getNewDappsAddresses (callback) {
    return this.addListener(this._api, 'parity_getDappDefaultAddress', (error, addresses) => {
      error
        ? callback(error)
        : callback(null, addresses ? addresses.map(outAddress) : null);
    });
  }

  getNewDappsDefaultAddress (callback) {
    return this.addListener(this._api, 'parity_getNewDappsDefaultAddress', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    });
  }

  listRecentDapps (callback) {
    return this.addListener(this._api, 'parity_listRecentDapps', (error, data) => {
      error
        ? callback(error)
        : callback(null, outRecentDapps(data));
    });
  }

  listGethAccounts (callback) {
    return this.addListener(this._api, 'parity_listGethAccounts', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddresses(data));
    });
  }

  listVaults (callback) {
    return this.addListener(this._api, 'parity_listVaults', callback);
  }

  listOpenedVaults (callback) {
    return this.addListener(this._api, 'parity_listOpenedVaults', callback);
  }

  getVaultMeta (callback, vaultName) {
    return this.addListener(this._api, 'parity_getVaultMeta', (error, data) => {
      error
        ? callback(error)
        : callback(null, outVaultMeta(data));
    }, [vaultName]);
  }

  deriveAddressHash (callback, address, password, hash, shouldSave) {
    return this.addListener(this._api, 'parity_deriveAddressHash', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    }, [inAddress(address), password, inDeriveHash(hash), !!shouldSave]);
  }

  deriveAddressIndex (callback, address, password, index, shouldSave) {
    return this.addListener(this._api, 'parity_deriveAddressIndex', (error, data) => {
      error
        ? callback(error)
        : callback(null, outAddress(data));
    }, [inAddress(address), password, inDeriveIndex(index), !!shouldSave]);
  }

  nodeHealth (callback) {
    return this.addListener(this._api, 'parity_nodeHealth', (error, data) => {
      error
        ? callback(error)
        : callback(null, data);
    });
  }
}
