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

import { inAddress, inAddresses, inBlockNumber, inData, inDeriveHash, inDeriveIndex, inHex, inNumber16, inOptions } from '../../format/input';
import { outAccountInfo, outAddress, outAddresses, outBlock, outChainStatus, outHistogram, outHwAccountInfo, outNodeKind, outNumber, outPeers, outRecentDapps, outTransaction, outVaultMeta } from '../../format/output';

export default class Parity {
  constructor (provider) {
    this._provider = provider;
  }

  acceptNonReservedPeers () {
    return this._provider
      .send('parity_acceptNonReservedPeers');
  }

  accountsInfo () {
    return this._provider
      .send('parity_accountsInfo')
      .then(outAccountInfo);
  }

  allAccountsInfo () {
    return this._provider
      .send('parity_allAccountsInfo')
      .then(outAccountInfo);
  }

  addReservedPeer (enode) {
    return this._provider
      .send('parity_addReservedPeer', enode);
  }

  chainStatus () {
    return this._provider
      .send('parity_chainStatus')
      .then(outChainStatus);
  }

  changePassword (account, password, newPassword) {
    return this._provider
      .send('parity_changePassword', inAddress(account), password, newPassword);
  }

  changeVault (account, vaultName) {
    return this._provider
      .send('parity_changeVault', inAddress(account), vaultName);
  }

  changeVaultPassword (vaultName, password) {
    return this._provider
      .send('parity_changeVaultPassword', vaultName, password);
  }

  checkRequest (requestId) {
    return this._provider
      .send('parity_checkRequest', inNumber16(requestId));
  }

  cidV0 (data) {
    return this._provider
      .send('parity_cidV0', inData(data));
  }

  closeVault (vaultName) {
    return this._provider
      .send('parity_closeVault', vaultName);
  }

  composeTransaction (options) {
    return this._provider
      .send('parity_composeTransaction', inOptions(options));
  }

  consensusCapability () {
    return this._provider
      .send('parity_consensusCapability');
  }

  dappsList () {
    return this._provider
      .send('parity_dappsList');
  }

  dappsUrl () {
    return this._provider
      .send('parity_dappsUrl');
  }

  decryptMessage (address, data) {
    return this._provider
      .send('parity_decryptMessage', inAddress(address), inHex(data));
  }

  defaultAccount () {
    return this._provider
      .send('parity_defaultAccount')
      .then(outAddress);
  }

  defaultExtraData () {
    return this._provider
      .send('parity_defaultExtraData');
  }

  devLogs () {
    return this._provider
      .send('parity_devLogs');
  }

  devLogsLevels () {
    return this._provider
      .send('parity_devLogsLevels');
  }

  deriveAddressHash (address, password, hash, shouldSave) {
    return this._provider
      .send('parity_deriveAddressHash', inAddress(address), password, inDeriveHash(hash), !!shouldSave)
      .then(outAddress);
  }

  deriveAddressIndex (address, password, index, shouldSave) {
    return this._provider
      .send('parity_deriveAddressIndex', inAddress(address), password, inDeriveIndex(index), !!shouldSave)
      .then(outAddress);
  }

  dropNonReservedPeers () {
    return this._provider
      .send('parity_dropNonReservedPeers');
  }

  enode () {
    return this._provider
      .send('parity_enode');
  }

  encryptMessage (pubkey, data) {
    return this._provider
      .send('parity_encryptMessage', inHex(pubkey), inHex(data));
  }

  executeUpgrade () {
    return this._provider
      .send('parity_executeUpgrade');
  }

  exportAccount (address, password) {
    return this._provider
      .send('parity_exportAccount', inAddress(address), password);
  }

  extraData () {
    return this._provider
      .send('parity_extraData');
  }

  futureTransactions () {
    return this._provider
      .send('parity_futureTransactions');
  }

  gasCeilTarget () {
    return this._provider
      .send('parity_gasCeilTarget')
      .then(outNumber);
  }

  gasFloorTarget () {
    return this._provider
      .send('parity_gasFloorTarget')
      .then(outNumber);
  }

  gasPriceHistogram () {
    return this._provider
      .send('parity_gasPriceHistogram')
      .then(outHistogram);
  }

  generateSecretPhrase () {
    return this._provider
      .send('parity_generateSecretPhrase');
  }

  getBlockHeaderByNumber (blockNumber = 'latest') {
    return this._provider
      .send('parity_getBlockHeaderByNumber', inBlockNumber(blockNumber))
      .then(outBlock);
  }

  getDappAddresses (dappId) {
    return this._provider
      .send('parity_getDappAddresses', dappId)
      .then(outAddresses);
  }

  getDappDefaultAddress (dappId) {
    return this._provider
      .send('parity_getDappDefaultAddress', dappId)
      .then(outAddress);
  }

  getNewDappsAddresses () {
    return this._provider
      .send('parity_getNewDappsAddresses')
      .then((addresses) => addresses ? addresses.map(outAddress) : null);
  }

  getNewDappsDefaultAddress () {
    return this._provider
      .send('parity_getNewDappsDefaultAddress')
      .then(outAddress);
  }

  getVaultMeta (vaultName) {
    return this._provider
      .send('parity_getVaultMeta', vaultName)
      .then(outVaultMeta);
  }

  hardwareAccountsInfo () {
    return this._provider
      .send('parity_hardwareAccountsInfo')
      .then(outHwAccountInfo);
  }

  hashContent (url) {
    return this._provider
      .send('parity_hashContent', url);
  }

  importGethAccounts (accounts) {
    return this._provider
      .send('parity_importGethAccounts', inAddresses(accounts))
      .then(outAddresses);
  }

  killAccount (account, password) {
    return this._provider
      .send('parity_killAccount', inAddress(account), password);
  }

  listAccounts (count, offset = null, blockNumber = 'latest') {
    return this._provider
      .send('parity_listAccounts', count, inAddress(offset), inBlockNumber(blockNumber))
      .then((accounts) => (accounts || []).map(outAddress));
  }

  listOpenedVaults () {
    return this._provider
      .send('parity_listOpenedVaults');
  }

  listVaults () {
    return this._provider
      .send('parity_listVaults');
  }

  listRecentDapps () {
    return this._provider
      .send('parity_listRecentDapps')
      .then(outRecentDapps);
  }

  listStorageKeys (address, count, hash = null, blockNumber = 'latest') {
    return this._provider
      .send('parity_listStorageKeys', inAddress(address), count, inHex(hash), inBlockNumber(blockNumber));
  }

  removeAddress (address) {
    return this._provider
      .send('parity_removeAddress', inAddress(address));
  }

  listGethAccounts () {
    return this._provider
      .send('parity_listGethAccounts')
      .then(outAddresses);
  }

  localTransactions () {
    return this._provider
      .send('parity_localTransactions')
      .then(transactions => {
        Object.values(transactions)
          .filter(tx => tx.transaction)
          .map(tx => {
            tx.transaction = outTransaction(tx.transaction);
          });
        return transactions;
      });
  }

  minGasPrice () {
    return this._provider
      .send('parity_minGasPrice')
      .then(outNumber);
  }

  mode () {
    return this._provider
      .send('parity_mode');
  }

  // DEPRECATED - use chain instead.
  netChain () {
    return this._provider
      .send('parity_chain');
  }

  nodeKind () {
    return this._provider
      .send('parity_nodeKind')
      .then(outNodeKind);
  }

  chain () {
    return this._provider
      .send('parity_chain');
  }

  netPeers () {
    return this._provider
      .send('parity_netPeers')
      .then(outPeers);
  }

  netMaxPeers () {
    return this._provider
      .send('parity_netMaxPeers')
      .then(outNumber);
  }

  netPort () {
    return this._provider
      .send('parity_netPort')
      .then(outNumber);
  }

  newAccountFromPhrase (phrase, password) {
    return this._provider
      .send('parity_newAccountFromPhrase', phrase, password)
      .then(outAddress);
  }

  newAccountFromSecret (secret, password) {
    return this._provider
      .send('parity_newAccountFromSecret', inHex(secret), password)
      .then(outAddress);
  }

  newAccountFromWallet (json, password) {
    return this._provider
      .send('parity_newAccountFromWallet', json, password)
      .then(outAddress);
  }

  newVault (vaultName, password) {
    return this._provider
      .send('parity_newVault', vaultName, password);
  }

  nextNonce (account) {
    return this._provider
      .send('parity_nextNonce', inAddress(account))
      .then(outNumber);
  }

  nodeName () {
    return this._provider
      .send('parity_nodeName');
  }

  openVault (vaultName, password) {
    return this._provider
      .send('parity_openVault', vaultName, password);
  }

  pendingTransactions () {
    return this._provider
      .send('parity_pendingTransactions')
      .then(data => data.map(outTransaction));
  }

  pendingTransactionsStats () {
    return this._provider
      .send('parity_pendingTransactionsStats');
  }

  phraseToAddress (phrase) {
    return this._provider
      .send('parity_phraseToAddress', phrase)
      .then(outAddress);
  }

  postSign (address, hash) {
    return this._provider
      .send('parity_postSign', inAddress(address), inHex(hash));
  }

  postTransaction (options = {}) {
    return this._provider
      .send('parity_postTransaction', inOptions(options));
  }

  registryAddress () {
    return this._provider
      .send('parity_registryAddress')
      .then(outAddress);
  }

  releasesInfo () {
    return this._provider
      .send('parity_releasesInfo');
  }

  removeReservedPeer (enode) {
    return this._provider
      .send('parity_removeReservedPeer', enode);
  }

  removeTransaction (hash) {
    return this._provider
      .send('parity_removeTransaction', inHex(hash))
      .then(outTransaction);
  }

  rpcSettings () {
    return this._provider
      .send('parity_rpcSettings');
  }

  setAccountName (address, name) {
    return this._provider
      .send('parity_setAccountName', inAddress(address), name);
  }

  setAccountMeta (address, meta) {
    return this._provider
      .send('parity_setAccountMeta', inAddress(address), JSON.stringify(meta));
  }

  setAuthor (address) {
    return this._provider
      .send('parity_setAuthor', inAddress(address));
  }

  setDappAddresses (dappId, addresses) {
    return this._provider
      .send('parity_setDappAddresses', dappId, inAddresses(addresses));
  }

  setDappDefaultAddress (dappId, address) {
    return this._provider
      .send('parity_setDappDefaultAddress', dappId, address ? inAddress(address) : null);
  }

  setEngineSigner (address, password) {
    return this._provider
      .send('parity_setEngineSigner', inAddress(address), password);
  }

  setExtraData (data) {
    return this._provider
      .send('parity_setExtraData', inData(data));
  }

  setGasCeilTarget (quantity) {
    return this._provider
      .send('parity_setGasCeilTarget', inNumber16(quantity));
  }

  setGasFloorTarget (quantity) {
    return this._provider
      .send('parity_setGasFloorTarget', inNumber16(quantity));
  }

  setMaxTransactionGas (quantity) {
    return this._provider
      .send('parity_setMaxTransactionGas', inNumber16(quantity));
  }

  setMinGasPrice (quantity) {
    return this._provider
      .send('parity_setMinGasPrice', inNumber16(quantity));
  }

  setMode (mode) {
    return this._provider
      .send('parity_setMode', mode);
  }

  setChain (specName) {
    return this._provider
      .send('parity_setChain', specName);
  }

  setNewDappsAddresses (addresses) {
    return this._provider
      .send('parity_setNewDappsAddresses', addresses ? inAddresses(addresses) : null);
  }

  setNewDappsDefaultAddress (address) {
    return this._provider
      .send('parity_setNewDappsDefaultAddress', inAddress(address));
  }

  setTransactionsLimit (quantity) {
    return this._provider
      .send('parity_setTransactionsLimit', inNumber16(quantity));
  }

  setVaultMeta (vaultName, meta) {
    return this._provider
      .send('parity_setVaultMeta', vaultName, JSON.stringify(meta));
  }

  signMessage (address, password, messageHash) {
    return this._provider
      .send('parity_signMessage', inAddress(address), password, inHex(messageHash));
  }

  testPassword (account, password) {
    return this._provider
      .send('parity_testPassword', inAddress(account), password);
  }

  transactionsLimit () {
    return this._provider
      .send('parity_transactionsLimit')
      .then(outNumber);
  }

  unsignedTransactionsCount () {
    return this._provider
      .send('parity_unsignedTransactionsCount')
      .then(outNumber);
  }

  upgradeReady () {
    return this._provider
      .send('parity_upgradeReady');
  }

  versionInfo () {
    return this._provider
      .send('parity_versionInfo');
  }

  wsUrl () {
    return this._provider
      .send('parity_wsUrl');
  }
}
