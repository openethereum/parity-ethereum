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

import { inAddress, inData, inHex, inNumber16, inOptions } from '../../format/input';
import { outAccountInfo, outAddress, outHistogram, outNumber, outPeers } from '../../format/output';

export default class Parity {
  constructor (transport) {
    this._transport = transport;
  }

  acceptNonReservedPeers () {
    return this._transport
      .execute('parity_acceptNonReservedPeers');
  }

  accounts () {
    return this._transport
      .execute('parity_accounts')
      .then(outAccountInfo);
  }

  accountsInfo () {
    return this._transport
      .execute('parity_accountsInfo')
      .then(outAccountInfo);
  }

  addReservedPeer (encode) {
    return this._transport
      .execute('parity_addReservedPeer', encode);
  }

  changePassword (account, password, newPassword) {
    return this._transport
      .execute('parity_changePassword', inAddress(account), password, newPassword);
  }

  checkRequest (requestId) {
    return this._transport
      .execute('parity_checkRequest', inNumber16(requestId));
  }

  dappsPort () {
    return this._transport
      .execute('parity_dappsPort')
      .then(outNumber);
  }

  dappsInterface () {
    return this._transport
      .execute('parity_dappsInterface');
  }

  defaultExtraData () {
    return this._transport
      .execute('parity_defaultExtraData');
  }

  devLogs () {
    return this._transport
      .execute('parity_devLogs');
  }

  devLogsLevels () {
    return this._transport
      .execute('parity_devLogsLevels');
  }

  dropNonReservedPeers () {
    return this._transport
      .execute('parity_dropNonReservedPeers');
  }

  enode () {
    return this._transport
      .execute('parity_enode');
  }

  extraData () {
    return this._transport
      .execute('parity_extraData');
  }

  gasFloorTarget () {
    return this._transport
      .execute('parity_gasFloorTarget')
      .then(outNumber);
  }

  gasPriceHistogram () {
    return this._transport
      .execute('parity_gasPriceHistogram')
      .then(outHistogram);
  }

  generateSecretPhrase () {
    return this._transport
      .execute('parity_generateSecretPhrase');
  }

  hashContent (url) {
    return this._transport
      .execute('parity_hashContent', url);
  }

  listGethAccounts () {
    return this._transport
      .execute('parity_listGethAccounts')
      .then((accounts) => (accounts || []).map(outAddress));
  }

  importGethAccounts (accounts) {
    return this._transport
      .execute('parity_importGethAccounts', (accounts || []).map(inAddress))
      .then((accounts) => (accounts || []).map(outAddress));
  }

  minGasPrice () {
    return this._transport
      .execute('parity_minGasPrice')
      .then(outNumber);
  }

  mode () {
    return this._transport
      .execute('parity_mode');
  }

  netChain () {
    return this._transport
      .execute('parity_netChain');
  }

  netPeers () {
    return this._transport
      .execute('parity_netPeers')
      .then(outPeers);
  }

  netMaxPeers () {
    return this._transport
      .execute('parity_netMaxPeers')
      .then(outNumber);
  }

  netPort () {
    return this._transport
      .execute('parity_netPort')
      .then(outNumber);
  }

  newAccountFromPhrase (phrase, password) {
    return this._transport
      .execute('parity_newAccountFromPhrase', phrase, password)
      .then(outAddress);
  }

  newAccountFromSecret (secret, password) {
    return this._transport
      .execute('parity_newAccountFromSecret', inHex(secret), password)
      .then(outAddress);
  }

  newAccountFromWallet (json, password) {
    return this._transport
      .execute('parity_newAccountFromWallet', json, password)
      .then(outAddress);
  }

  nextNonce (account) {
    return this._transport
      .execute('parity_nextNonce', inAddress(account))
      .then(outNumber);
  }

  nodeName () {
    return this._transport
      .execute('parity_nodeName');
  }

  phraseToAddress (phrase) {
    return this._transport
      .execute('parity_phraseToAddress', phrase)
      .then(outAddress);
  }

  postTransaction (options) {
    return this._transport
      .execute('parity_postTransaction', inOptions(options));
  }

  registryAddress () {
    return this._transport
      .execute('parity_registryAddress')
      .then(outAddress);
  }

  removeReservedPeer (encode) {
    return this._transport
      .execute('parity_removeReservedPeer', encode);
  }

  rpcSettings () {
    return this._transport
      .execute('parity_rpcSettings');
  }

  setAccountName (address, name) {
    return this._transport
      .execute('parity_setAccountName', inAddress(address), name);
  }

  setAccountMeta (address, meta) {
    return this._transport
      .execute('parity_setAccountMeta', inAddress(address), JSON.stringify(meta));
  }

  setAuthor (address) {
    return this._transport
      .execute('parity_setAuthor', inAddress(address));
  }

  setExtraData (data) {
    return this._transport
      .execute('parity_setExtraData', inData(data));
  }

  setGasFloorTarget (quantity) {
    return this._transport
      .execute('parity_setGasFloorTarget', inNumber16(quantity));
  }

  setMinGasPrice (quantity) {
    return this._transport
      .execute('parity_setMinGasPrice', inNumber16(quantity));
  }

  setMode (mode) {
    return this._transport
      .execute('parity_setMode', mode);
  }

  setTransactionsLimit (quantity) {
    return this._transport
      .execute('parity_setTransactionsLimit', inNumber16(quantity));
  }

  signerPort () {
    return this._transport
      .execute('parity_signerPort')
      .then(outNumber);
  }

  testPassword (account, password) {
    return this._transport
      .execute('parity_testPassword', inAddress(account), password);
  }

  transactionsLimit () {
    return this._transport
      .execute('parity_transactionsLimit')
      .then(outNumber);
  }

  unsignedTransactionsCount () {
    return this._transport
      .execute('parity_unsignedTransactionsCount')
      .then(outNumber);
  }
}
