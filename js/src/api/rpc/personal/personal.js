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

import { inAddress, inNumber10, inNumber16, inOptions } from '../../format/input';
import { outAccountInfo, outAddress, outSignerRequest } from '../../format/output';

export default class Personal {
  constructor (transport) {
    this._transport = transport;
  }

  accountsInfo () {
    return this._transport
      .execute('personal_accountsInfo')
      .then(outAccountInfo);
  }

  confirmRequest (requestId, options, password) {
    return this._transport
      .execute('personal_confirmRequest', inNumber16(requestId), options, password);
  }

  generateAuthorizationToken () {
    return this._transport
      .execute('personal_generateAuthorizationToken');
  }

  listAccounts () {
    return this._transport
      .execute('personal_listAccounts')
      .then((accounts) => (accounts || []).map(outAddress));
  }

  listGethAccounts () {
    return this._transport
      .execute('personal_listGethAccounts')
      .then((accounts) => (accounts || []).map(outAddress));
  }

  importGethAccounts (accounts) {
    return this._transport
      .execute('personal_importGethAccounts', (accounts || []).map(inAddress))
      .then((accounts) => (accounts || []).map(outAddress));
  }

  newAccount (password) {
    return this._transport
      .execute('personal_newAccount', password)
      .then(outAddress);
  }

  newAccountFromPhrase (phrase, password) {
    return this._transport
      .execute('personal_newAccountFromPhrase', phrase, password)
      .then(outAddress);
  }

  newAccountFromWallet (json, password) {
    return this._transport
      .execute('personal_newAccountFromWallet', json, password)
      .then(outAddress);
  }

  rejectRequest (requestId) {
    return this._transport
      .execute('personal_rejectRequest', inNumber16(requestId));
  }

  requestsToConfirm () {
    return this._transport
      .execute('personal_requestsToConfirm')
      .then((requests) => (requests || []).map(outSignerRequest));
  }

  setAccountName (address, name) {
    return this._transport
      .execute('personal_setAccountName', inAddress(address), name);
  }

  setAccountMeta (address, meta) {
    return this._transport
      .execute('personal_setAccountMeta', inAddress(address), JSON.stringify(meta));
  }

  signAndSendTransaction (options, password) {
    return this._transport
      .execute('personal_signAndSendTransaction', inOptions(options), password);
  }

  signerEnabled () {
    return this._transport
      .execute('personal_signerEnabled');
  }

  unlockAccount (account, password, duration = 1) {
    return this._transport
      .execute('personal_unlockAccount', inAddress(account), password, inNumber10(duration));
  }
}
