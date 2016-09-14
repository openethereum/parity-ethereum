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

import { handleActions } from 'redux-actions';

const initialState = {
};

export default handleActions({
  personalAccountsInfo (state, action) {
    const { accountsInfo } = action;
    const accounts = {};
    const contacts = {};

    Object.keys(accountsInfo).forEach((address) => {
      const account = accountsInfo[address];
      const { name, meta, uuid } = account;

      if (uuid) {
        accounts[address] = { address, name, meta, uuid };
      } else {
        contacts[address] = { address, name, meta };
      }
    });

    return Object.assign({}, state, {
      accounts,
      hasAccounts: Object.keys(accounts).length !== 0,
      contacts,
      hasContacts: Object.keys(contacts).length !== 0
    });
  }
}, initialState);
