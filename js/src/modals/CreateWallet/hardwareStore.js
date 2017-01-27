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

import Ledger from '~/3rdparty/ledger';

export default class HardwareStore {
  constructor (api) {
    this._api = api;
    this._ledger = Ledger.create();
  }

  scan () {
    return this._ledger
      .scan()
      .then((response) => {
        console.log('HardwareStore::scan', response);
        return this.createEntry(response.address, 'Ledger Nano', 'Ledger hardware wallet', 'ledger');
      })
      .catch((error) => {
        console.wran('HardwareStore::scan', error);
      });
  }

  createEntry (address, name, description, type) {
    return Promise
      .all([
        this._api.setAccountName(address, name),
        this._api.setAccountMeta(address, {
          description,
          hardware: { type }
        })
      ])
      .catch((error) => {
        console.warn('HardwareStore::createEntry', error);
        throw error;
      });
  }
}
