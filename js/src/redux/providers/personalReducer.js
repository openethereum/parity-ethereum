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

import { handleActions } from 'redux-actions';
import { isEqual } from 'lodash';

const initialState = {
  accountsInfo: {},
  accounts: {},
  hasAccounts: false,
  contacts: {},
  hasContacts: false,
  contracts: {},
  hasContracts: false,
  visibleAccounts: []
};

export default handleActions({
  personalAccountsInfo (state, action) {
    const accountsInfo = action.accountsInfo || state.accountsInfo;
    const { accounts, contacts, contracts } = action;

    return Object.assign({}, state, {
      accountsInfo,
      accounts,
      hasAccounts: Object.keys(accounts).length !== 0,
      contacts,
      hasContacts: Object.keys(contacts).length !== 0,
      contracts,
      hasContracts: Object.keys(contracts).length !== 0
    });
  },

  setVisibleAccounts (state, action) {
    const addresses = (action.addresses || []).sort();

    if (isEqual(addresses, state.visibleAccounts)) {
      return state;
    }

    return Object.assign({}, state, {
      visibleAccounts: addresses
    });
  }
}, initialState);
