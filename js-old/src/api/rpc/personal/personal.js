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

import { inAddress, inNumber10, inOptions } from '../../format/input';
import { outAddress } from '../../format/output';

export default class Personal {
  constructor (transport) {
    this._transport = transport;
  }

  listAccounts () {
    return this._transport
      .execute('personal_listAccounts')
      .then((accounts) => (accounts || []).map(outAddress));
  }

  newAccount (password) {
    return this._transport
      .execute('personal_newAccount', password)
      .then(outAddress);
  }

  sendTransaction (options, password) {
    return this._transport
      .execute('personal_sendTransaction', inOptions(options), password);
  }

  unlockAccount (account, password, duration = 1) {
    return this._transport
      .execute('personal_unlockAccount', inAddress(account), password, inNumber10(duration));
  }
}
