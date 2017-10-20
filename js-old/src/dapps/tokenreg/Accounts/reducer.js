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

import {
  SET_ACCOUNTS,
  SET_SELECTED_ACCOUNT,
  SET_ACCOUNTS_INFO
} from './actions';

const initialState = {
  list: [],
  accountsInfo: {},
  selected: null
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_ACCOUNTS:
      return {
        ...state,
        list: [].concat(action.accounts)
      };

    case SET_ACCOUNTS_INFO:
      return {
        ...state,
        accountsInfo: { ...action.accountsInfo }
      };

    case SET_SELECTED_ACCOUNT: {
      const address = action.address;
      const account = state.list.find(a => a.address === address);

      return {
        ...state,
        selected: account
      };
    }

    default:
      return state;
  }
};

