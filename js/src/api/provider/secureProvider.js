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

import { inAddress, inDeriveHash, inDeriveIndex } from '../format/input';
import { outAccountInfo, outAddress, outAddresses, outRecentDapps, outVaultMeta } from '../format/output';

import Provider from './provider';

export default class SecureProvider extends Provider {
  constructor (transport) {
    if (!transport.isSecure) {
      throw Error('Can`t provide secure API without secure transport!');
    }
    super(transport);
    this._api = 'parity_subscribe';
  }

  unsubscribe (...subscriptionIds) {
    return this._removeListener('parity_unsubscribe', subscriptionIds);
  }

  // parity accounts API (only secure API)

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
