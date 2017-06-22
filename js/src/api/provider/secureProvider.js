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
    super(transport);
    this._api = 'parity';
  }

  unsubscribe (...subscriptionIds) {
    return this._removeListener('parity', subscriptionIds);
  }

  // parity accounts API (only secure API)
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
