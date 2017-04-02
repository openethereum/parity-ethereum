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

import { api } from '../parity';

export const SET_ACCOUNTS = 'SET_ACCOUNTS';
export const setAccounts = (accounts) => ({
  type: SET_ACCOUNTS,
  accounts
});

export const SET_ACCOUNTS_INFO = 'SET_ACCOUNTS_INFO';
export const setAccountsInfo = (accountsInfo) => ({
  type: SET_ACCOUNTS_INFO,
  accountsInfo
});

export const SET_SELECTED_ACCOUNT = 'SET_SELECTED_ACCOUNT';
export const setSelectedAccount = (address) => ({
  type: SET_SELECTED_ACCOUNT,
  address
});

export const loadAccounts = () => (dispatch) => {
  api.parity
    .accountsInfo()
    .then((accountsInfo) => {
      const accountsList = Object
        .keys(accountsInfo)
        .map((address) => ({
          ...accountsInfo[address],
          address
        }));

      dispatch(setAccounts(accountsList));
      dispatch(setAccountsInfo(accountsInfo));
      dispatch(setSelectedAccount(accountsList[0].address));
    })
    .catch(e => {
      console.error('loadAccounts error', e);
    });
};
